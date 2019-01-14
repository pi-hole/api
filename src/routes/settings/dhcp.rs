// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// DHCP Configuration Settings
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    routes::{auth::User, settings::common::restart_dns},
    settings::{generate_dnsmasq_config, ConfigEntry, SetupVarsEntry},
    util::{reply_data, reply_success, Error, ErrorKind, Reply}
};
use rocket::State;
use rocket_contrib::json::Json;

#[derive(Serialize, Deserialize)]
pub struct DhcpSettings {
    active: bool,
    ip_start: String,
    ip_end: String,
    router_ip: String,
    lease_time: usize,
    domain: String,
    ipv6_support: bool
}

impl DhcpSettings {
    /// Check if all settings are valid
    fn is_valid(&self) -> bool {
        // If DHCP is to be turned on, no settings may be empty
        if self.active
            && (self.ip_start.is_empty()
                || self.ip_end.is_empty()
                || self.router_ip.is_empty()
                || self.domain.is_empty())
        {
            return false;
        }

        SetupVarsEntry::DhcpStart.is_valid(&self.ip_start)
            && SetupVarsEntry::DhcpEnd.is_valid(&self.ip_end)
            && SetupVarsEntry::DhcpRouter.is_valid(&self.router_ip)
            && SetupVarsEntry::PiholeDomain.is_valid(&self.domain)
    }
}

/// Get DHCP Configuration
#[get("/settings/dhcp")]
pub fn get_dhcp(env: State<Env>, _auth: User) -> Reply {
    let dhcp_settings = DhcpSettings {
        active: SetupVarsEntry::DhcpActive.is_true(&env)?,
        ip_start: SetupVarsEntry::DhcpStart.read(&env)?,
        ip_end: SetupVarsEntry::DhcpEnd.read(&env)?,
        router_ip: SetupVarsEntry::DhcpRouter.read(&env)?,
        lease_time: SetupVarsEntry::DhcpLeasetime.read_as(&env)?,
        domain: SetupVarsEntry::PiholeDomain.read(&env)?,
        ipv6_support: SetupVarsEntry::DhcpIpv6.is_true(&env)?
    };

    reply_data(dhcp_settings)
}

/// Update DHCP Configuration
#[put("/settings/dhcp", data = "<data>")]
pub fn put_dhcp(env: State<Env>, _auth: User, data: Json<DhcpSettings>) -> Reply {
    let settings: DhcpSettings = data.into_inner();

    if !settings.is_valid() {
        return Err(Error::from(ErrorKind::InvalidSettingValue));
    }

    SetupVarsEntry::DhcpActive.write(&settings.active.to_string(), &env)?;
    SetupVarsEntry::DhcpStart.write(&settings.ip_start, &env)?;
    SetupVarsEntry::DhcpEnd.write(&settings.ip_end, &env)?;
    SetupVarsEntry::DhcpRouter.write(&settings.router_ip, &env)?;
    SetupVarsEntry::DhcpLeasetime.write(&settings.lease_time.to_string(), &env)?;
    SetupVarsEntry::PiholeDomain.write(&settings.domain, &env)?;
    SetupVarsEntry::DhcpIpv6.write(&settings.ipv6_support.to_string(), &env)?;

    generate_dnsmasq_config(&env)?;
    restart_dns(&env)?;
    reply_success()
}

#[cfg(test)]
mod test {
    use crate::{env::PiholeFile, routes::settings::dhcp::DhcpSettings, testing::TestBuilder};
    use rocket::http::Method;

    /// Verify that having active DHCP and missing settings is invalid
    #[test]
    fn invalid_if_active_and_empty() {
        let settings = DhcpSettings {
            active: true,
            ip_start: "".to_owned(),
            ip_end: "".to_owned(),
            router_ip: "".to_owned(),
            lease_time: 24,
            domain: "".to_owned(),
            ipv6_support: false
        };

        assert_eq!(settings.is_valid(), false);
    }

    /// Verify that having inactive DHCP and missing settings is valid
    #[test]
    fn valid_if_inactive_and_empty() {
        let settings = DhcpSettings {
            active: false,
            ip_start: "".to_owned(),
            ip_end: "".to_owned(),
            router_ip: "".to_owned(),
            lease_time: 24,
            domain: "".to_owned(),
            ipv6_support: false
        };

        assert_eq!(settings.is_valid(), true);
    }

    /// Verify that having active DHCP and no missing settings is valid
    #[test]
    fn valid_if_active_and_not_empty() {
        let settings = DhcpSettings {
            active: false,
            ip_start: "192.168.1.50".to_owned(),
            ip_end: "192.168.1.150".to_owned(),
            router_ip: "192.168.1.1".to_owned(),
            lease_time: 24,
            domain: "lan".to_owned(),
            ipv6_support: false
        };

        assert_eq!(settings.is_valid(), true);
    }

    /// Verify that having invalid settings is invalid
    #[test]
    fn invalid_if_setting_invalid() {
        let settings = DhcpSettings {
            active: false,
            ip_start: "not an IP".to_owned(),
            ip_end: "not an IP".to_owned(),
            router_ip: "not an IP".to_owned(),
            lease_time: 24,
            domain: "not a domain".to_owned(),
            ipv6_support: false
        };

        assert_eq!(settings.is_valid(), false);
    }

    /// Basic test for stored settings
    #[test]
    fn get_full_setup() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dhcp")
            .file(
                PiholeFile::SetupVars,
                "DHCP_START=192.168.1.201\n\
                 DHCP_END=192.168.1.251\n\
                 DHCP_ROUTER=192.168.1.1\n\
                 DHCP_LEASETIME=24\n\
                 PIHOLE_DOMAIN=lan\n\
                 DHCP_IPv6=false\n\
                 DHCP_ACTIVE=false\n"
            )
            .expect_json(json!({
                "active": false,
                "ip_start": "192.168.1.201",
                "ip_end": "192.168.1.251",
                "router_ip": "192.168.1.1",
                "lease_time": 24,
                "domain": "lan",
                "ipv6_support": false,
            }))
            .test();
    }

    /// Test that default settings are returned if not present
    #[test]
    fn get_minimal_setup() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dhcp")
            .file(PiholeFile::SetupVars, "")
            .expect_json(json!({
                "active": false,
                "ip_start": "",
                "ip_end": "",
                "router_ip": "",
                "lease_time": 24,
                "domain": "",
                "ipv6_support": false,
            }))
            .test();
    }

    /// Updating with new settings should store the settings and generate a new
    /// config
    #[test]
    fn put_dhcp() {
        TestBuilder::new()
            .endpoint("/admin/api/settings/dhcp")
            .method(Method::Put)
            .file_expect(
                PiholeFile::SetupVars,
                "PIHOLE_DNS_1=8.8.8.8\n\
                 PIHOLE_INTERFACE=eth0\n",
                "PIHOLE_DNS_1=8.8.8.8\n\
                 PIHOLE_INTERFACE=eth0\n\
                 DHCP_ACTIVE=true\n\
                 DHCP_START=192.168.1.50\n\
                 DHCP_END=192.168.1.150\n\
                 DHCP_ROUTER=192.168.1.1\n\
                 DHCP_LEASETIME=24\n\
                 PIHOLE_DOMAIN=lan\n\
                 DHCP_IPv6=true\n"
            )
            .file_expect(
                PiholeFile::DnsmasqConfig,
                "",
                "################################################################\n\
                 #       THIS FILE IS AUTOMATICALLY GENERATED BY PI-HOLE.       #\n\
                 #          ANY CHANGES MADE TO THIS FILE WILL BE LOST.         #\n\
                 #                                                              #\n\
                 #  NEW CONFIG SETTINGS MUST BE MADE IN A SEPARATE CONFIG FILE  #\n\
                 #                OR IN /etc/dnsmasq.conf                       #\n\
                 ################################################################\n\
                 \n\
                 localise-queries\n\
                 local-ttl=2\n\
                 cache-size=10000\n\
                 server=8.8.8.8\n\
                 addn-hosts=/etc/pihole/gravity.list\n\
                 addn-hosts=/etc/pihole/black.list\n\
                 addn-hosts=/etc/pihole/local.list\n\
                 domain-needed\n\
                 bogus-priv\n\
                 local-service\n\
                 dhcp-authoritative\n\
                 dhcp-leasefile=/etc/pihole/dhcp.leases\n\
                 dhcp-range=192.168.1.50,192.168.1.150,24h\n\
                 dhcp-option=option:router,192.168.1.1\n\
                 dhcp-name-match=set:wpad-ignore,wpad\n\
                 dhcp-ignore-names=tag:wpad-ignore\n\
                 dhcp-option=option6:dns-server,[::]\n\
                 dhcp-range=::100,::1ff,constructor:eth0,ra-names,slaac,24h\n\
                 ra-param=*,0,0\n"
            )
            .body(json!({
                "active": true,
                "ip_start": "192.168.1.50",
                "ip_end": "192.168.1.150",
                "router_ip": "192.168.1.1",
                "lease_time": 24,
                "domain": "lan",
                "ipv6_support": true
            }))
            .expect_json(json!({
                "status": "success"
            }))
            .test();
    }
}

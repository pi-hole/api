Name:           pihole-api
Version:        0.1.0
Release:        1
Summary:        The Pi-hole API, including the Web Interface

License:        EUPL-1.2
URL:            https://pi-hole.net

BuildRequires:  systemd
Requires:       libcap
%{?systemd_requires}


%description
The Pi-hole API provides a RESTful service for the web interface. The web
interface is embedded into the API and exposed under /admin.


%prep
cp -r %{_sourcedir}/* %{_builddir}


%install
rm -rf $RPM_BUILD_ROOT
%make_install
mkdir -p %{buildroot}%{_unitdir}
install -m 644 debian/pihole-API.service %{buildroot}%{_unitdir}


%files
%license LICENSE
%{_bindir}/pihole-API
%{_unitdir}/pihole-API.service


%post
# Only add the user when installing
if [ $1 -eq 1 ]; then
    # Create a pihole user and group if they don't already exist
    adduser --system --user-group pihole &>/dev/null
fi

# Give the API permission to bind to low ports
setcap CAP_NET_BIND_SERVICE+eip /usr/bin/pihole-API

%systemd_post pihole-API.service


%preun
%systemd_preun pihole-API.service


%postun
%systemd_postun_with_restart pihole-API.service


%changelog
* Fri Mar 22 2019 Mark Drobnak <mark.drobnak@pi-hole.net> - 0.1.0-1
- Initial package
// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Web Interface Settings Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::Env,
    routes::auth::User,
    settings::{ConfigEntry, SetupVarsEntry},
    util::{reply_data, reply_success, Error, ErrorKind, Reply}
};
use rocket::State;
use rocket_contrib::json::Json;

/// Get web interface settings
#[get("/settings/web")]
pub fn get_web(env: State<Env>) -> Reply {
    let settings = WebSettings {
        layout: SetupVarsEntry::WebLayout.read(&env)?,
        language: SetupVarsEntry::WebLanguage.read(&env)?
    };

    reply_data(settings)
}

/// Update web interface settings
#[put("/settings/web", data = "<settings>")]
pub fn put_web(_auth: User, env: State<Env>, settings: Json<WebSettings>) -> Reply {
    let settings = settings.into_inner();

    if !settings.is_valid() {
        return Err(Error::from(ErrorKind::InvalidSettingValue));
    }

    SetupVarsEntry::WebLayout.write(&settings.layout, &env)?;
    SetupVarsEntry::WebLanguage.write(&settings.language, &env)?;

    reply_success()
}

#[derive(Serialize, Deserialize)]
pub struct WebSettings {
    layout: String,
    language: String
}

impl WebSettings {
    /// Check if all the web settings are valid
    fn is_valid(&self) -> bool {
        SetupVarsEntry::WebLayout.is_valid(&self.layout)
            && SetupVarsEntry::WebLanguage.is_valid(&self.language)
    }
}

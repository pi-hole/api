// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Gravity Database Schema
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

table! {
    adlists (id) {
        id -> Integer,
        address -> Text,
        enabled -> Bool,
        date_added -> Integer,
        date_modified -> Integer,
        comment -> Nullable<Text>,
    }
}

table! {
    blacklist (id) {
        id -> Integer,
        domain -> Text,
        enabled -> Bool,
        date_added -> Integer,
        date_modified -> Integer,
        comment -> Nullable<Text>,
    }
}

table! {
    gravity (domain) {
        domain -> Text,
    }
}

table! {
    info (property) {
        property -> Text,
        value -> Text,
    }
}

table! {
    regex (id) {
        id -> Integer,
        domain -> Text,
        enabled -> Bool,
        date_added -> Integer,
        date_modified -> Integer,
        comment -> Nullable<Text>,
    }
}

table! {
    whitelist (id) {
        id -> Integer,
        domain -> Text,
        enabled -> Bool,
        date_added -> Integer,
        date_modified -> Integer,
        comment -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(adlists, blacklist, gravity, info, regex, whitelist,);

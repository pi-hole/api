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
    adlist (id) {
        id -> Integer,
        address -> Text,
        enabled -> Bool,
        date_added -> Integer,
        date_modified -> Integer,
        comment -> Nullable<Text>,
    }
}

table! {
    adlist_by_group (adlist_id, group_id) {
        adlist_id -> Integer,
        group_id -> Integer,
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
    blacklist_by_group (blacklist_id, group_id) {
        blacklist_id -> Integer,
        group_id -> Integer,
    }
}

table! {
    gravity (domain) {
        domain -> Text,
    }
}

table! {
    group (id) {
        id -> Integer,
        enabled -> Bool,
        name -> Text,
        description -> Nullable<Text>,
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
    regex_by_group (regex_id, group_id) {
        regex_id -> Integer,
        group_id -> Integer,
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

table! {
    whitelist_by_group (whitelist_id, group_id) {
        whitelist_id -> Integer,
        group_id -> Integer,
    }
}

joinable!(adlist_by_group -> adlist (adlist_id));
joinable!(adlist_by_group -> group (group_id));
joinable!(blacklist_by_group -> blacklist (blacklist_id));
joinable!(blacklist_by_group -> group (group_id));
joinable!(regex_by_group -> group (group_id));
joinable!(regex_by_group -> regex (regex_id));
joinable!(whitelist_by_group -> group (group_id));
joinable!(whitelist_by_group -> whitelist (whitelist_id));

allow_tables_to_appear_in_same_query!(
    adlist,
    adlist_by_group,
    blacklist,
    blacklist_by_group,
    gravity,
    group,
    info,
    regex,
    regex_by_group,
    whitelist,
    whitelist_by_group,
);

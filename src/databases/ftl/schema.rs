// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// FTL Database Schema
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

table! {
    counters (id) {
        id -> Integer,
        value -> Integer,
    }
}

table! {
    ftl (id) {
        id -> Integer,
        value -> Binary,
    }
}

table! {
    network (id) {
        id -> Integer,
        ip -> Text,
        hwaddr -> Text,
        interface -> Text,
        name -> Nullable<Text>,
        firstSeen -> Integer,
        lastQuery -> Integer,
        numQueries -> Integer,
        macVendor -> Nullable<Text>,
    }
}

table! {
    queries (id) {
        id -> Nullable<Integer>,
        timestamp -> Integer,
        #[sql_name = "type"]
        query_type -> Integer,
        status -> Integer,
        domain -> Text,
        client -> Text,
        forward -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(counters, ftl, network, queries,);

table! {
    measurements (id) {
        id -> Int4,
        timestamp -> Timestamptz,
        sensor_id -> Int4,
        temperature -> Nullable<Float8>,
        pressure -> Nullable<Float8>,
    }
}

table! {
    sensors (id) {
        id -> Int4,
        description -> Text,
    }
}

joinable!(measurements -> sensors (sensor_id));

allow_tables_to_appear_in_same_query!(
    measurements,
    sensors,
);

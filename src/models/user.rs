use std::fmt;
use std::fmt::{Debug, Display};
use std::time::SystemTime;

use diesel::sql_types::{Uuid as SqlUuid, VarChar};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "SqlUuid"]
pub struct UserId(Uuid);
derive_newtype_sql!(user_id, SqlUuid, UserId, UserId);

#[derive(Deserialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "VarChar"]
pub struct AuthenticationToken(String);
derive_newtype_sql!(authentication_token, VarChar, AuthenticationToken, AuthenticationToken);
mask_logs!(AuthenticationToken);

#[derive(Debug, Deserialize, Queryable, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: UserId,
    name: String,
    authentication_token: AuthenticationToken,
    created_at: SystemTime,
    updated_at: SystemTime,
}

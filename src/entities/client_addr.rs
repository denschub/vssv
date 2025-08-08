use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts},
    http::{HeaderMap, request::Parts},
};

use crate::{AppState, errors::ResponseError};

/// An access token stored in the database.
#[derive(Debug)]
pub struct ClientAddr {
    pub ip: IpAddr,
}

#[derive(Debug)]
pub struct ExtractClientAddr(pub ClientAddr);

impl<S> FromRequestParts<S> for ExtractClientAddr
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ResponseError;

    /// This [FromRequestParts] implementation tries to read the x-real-ip, but
    /// falls back to [ConnectInfo<SocketAddr>] if the setting is not turned on
    /// or if the x-real-ip header is missing. If the header is there, but
    /// somehow malformed, it will reject and return a 400.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let server_state = AppState::from_ref(state);
        if server_state.settings.use_x_real_ip {
            let headers = HeaderMap::from_request_parts(parts, state)
                .await
                .expect("HeaderMap::from_request_parts to be infallible");

            if let Some(header) = headers.get("x-real-ip") {
                let header_val = header.to_str()?;
                let ip = IpAddr::from_str(header_val)?;
                return Ok(Self(ClientAddr { ip }));
            }
        }

        let conn_info: ConnectInfo<SocketAddr> =
            ConnectInfo::from_request_parts(parts, state).await?;
        Ok(Self(ClientAddr { ip: conn_info.ip() }))
    }
}

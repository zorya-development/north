use leptos::prelude::*;

#[server(Ping, "/api")]
pub async fn ping() -> Result<(), ServerFnError> {
    Ok(())
}

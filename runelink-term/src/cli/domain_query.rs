use runelink_client::util::get_api_url;
use uuid::Uuid;

use crate::{error::CliError, storage::TryGetDomain};

use super::context::CliContext;

#[allow(dead_code)]
pub struct DomainQueryBuilder<'a> {
    ctx: &'a CliContext<'a>,
    domain: Option<String>,
    server_id: Option<Uuid>,
}

#[allow(dead_code)]
impl<'a> DomainQueryBuilder<'a> {
    pub fn new(ctx: &'a CliContext<'a>) -> Self {
        DomainQueryBuilder {
            ctx,
            domain: None,
            server_id: None,
        }
    }

    pub fn try_domain(mut self, domain: Option<String>) -> Self {
        self.domain = domain;
        self
    }

    pub fn try_server(mut self, server_id: Option<Uuid>) -> Self {
        self.server_id = server_id;
        self
    }

    pub fn get_domain(&self) -> Result<String, CliError> {
        if let Some(domain) = &self.domain {
            return Ok(domain.into());
        }
        if let Some(server_id) = self.server_id {
            return match self.ctx.config.try_get_server_api_url(server_id) {
                Ok(server_domain) => Ok(server_domain),
                Err(_) => self
                    .ctx
                    .account
                    .try_get_domain()
                    .map_err(|_| {
                        CliError::MissingContext(
                            "Could not determine request domain.".into(),
                        )
                    })
                    .map(|domain| domain.to_string()),
            };
        }
        self.ctx
            .account
            .try_get_domain()
            .map_err(|_| {
                CliError::MissingContext(
                    "Could not determine request domain.".into(),
                )
            })
            .map(|domain| domain.to_string())
    }

    pub fn get_api_url(&self) -> Result<String, CliError> {
        let domain = self.get_domain()?;
        Ok(get_api_url(&domain))
    }

    pub fn get_domain_and_api_url(&self) -> Result<(String, String), CliError> {
        let domain = self.get_domain()?;
        let api_url = get_api_url(&domain);
        Ok((domain, api_url))
    }
}

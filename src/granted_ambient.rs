// Copyright 2026 Mark AB (markik)
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Read-only neighboring context through the resident's per-app, per-mere
//! ambient grant. No shared-root files, complete graph, media, or write door.
use chirograph::{PortableCardV1, PresentationCodec, ProjectionRequest};
use djinn::door::app_admission::{AppId, AppRouteId};
use graphshell_endpoint::door::projection::AppBrokerClient;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrantedCard {
    pub domain: String,
    pub resource: String,
    pub card: PortableCardV1,
}
pub struct GrantedAmbient {
    client: AppBrokerClient,
    request: ProjectionRequest,
    domain: String,
    cards: Vec<GrantedCard>,
}
impl GrantedAmbient {
    pub async fn open_at(endpoint: &str, domain: &str) -> Result<Self, String> {
        let domain = pandect::DomainId::new(domain)
            .map_err(|e| e.to_string())?
            .to_string();
        let route = AppRouteId::new(mere_resident::resident_mere::ambient_mere_route_id(&domain))
            .map_err(|e| e.to_string())?;
        let mut client = AppBrokerClient::open_route_at(endpoint, AppId::new("cleromancy"), route)
            .await
            .map_err(|e| e.to_string())?;
        let opened = client.open_session().await.map_err(|e| e.to_string())?;
        let request = opened
            .descriptor
            .projections
            .iter()
            .find(|p| p.request.session.0 == mere_overmap::wire::MERE_GRAPH)
            .map(|p| p.request.clone())
            .ok_or("ambient route has no graph projection")?;
        Ok(Self {
            client,
            request,
            domain,
            cards: Vec::new(),
        })
    }
    pub fn cards(&self) -> &[GrantedCard] {
        &self.cards
    }
    pub async fn refresh(&mut self) -> Result<bool, String> {
        let result = self.read().await;
        match result {
            Ok(cards) => {
                let changed = self.cards != cards;
                self.cards = cards;
                Ok(changed)
            },
            Err(error) => {
                self.cards.clear();
                Err(error)
            },
        }
    }
    async fn read(&mut self) -> Result<Vec<GrantedCard>, String> {
        // Every refresh and resource read crosses the live resident guard. A
        // revoked grant never leaves the previous cards in this read model.
        let snapshot = self
            .client
            .snapshot(self.request.clone())
            .await
            .map_err(|e| e.to_string())?;
        let mut cards = Vec::new();
        for binding in &snapshot.presentation.bindings {
            if binding.instance == sceno::InstanceId(0) {
                continue;
            }
            let Some(offers) = snapshot.presentation.offers.get(&binding.key) else {
                continue;
            };
            for offer in offers {
                if cards.len() >= 16 {
                    break;
                }
                if offer.codec != PresentationCodec::PortableCardV1 || offer.byte_size > 64 * 1024 {
                    continue;
                }
                let bytes = self
                    .client
                    .resource(snapshot.session.clone(), offer.resource)
                    .await
                    .map_err(|e| e.to_string())?;
                if bytes.len() > 64 * 1024 {
                    return Err("ambient card exceeds advertised bounds".into());
                }
                let mut card: PortableCardV1 =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                card.media.clear(); // this product does not request neighboring media
                cards.push(GrantedCard {
                    domain: self.domain.clone(),
                    resource: offer.resource.to_string(),
                    card,
                });
                break;
            }
        }
        Ok(cards)
    }
    pub async fn close(self) -> Result<(), String> {
        self.client.close().await.map_err(|e| e.to_string())
    }
}

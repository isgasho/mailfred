pub mod filters;
pub mod layers;

use async_trait::async_trait;

use crate::service::{
    response::{Response, ResponseResult},
    Request, Service,
};

pub trait Filter: Send + Sync + 'static {
    fn check(&self, header: &str) -> bool;
}

pub trait Layer: Send + Sync + 'static {
    fn map_request(&self, request: Request) -> Request {
        request
    }

    fn map_response(&self, response: ResponseResult) -> ResponseResult {
        response
    }
}

pub struct Route<F, S> {
    filter: F,
    service: S,
}

#[async_trait]
impl<State, F, S> Service<State> for Route<F, S>
where
    State: Send + 'static,
    F: Send + Sync + 'static,
    S: Service<State>,
{
    async fn call(&self, request: Request, state: State) -> ResponseResult {
        self.service.call(request, state).await.into()
    }
}

impl<F, S> Filter for Route<F, S>
where
    F: Filter,
    S: Send + Sync + 'static,
{
    fn check(&self, header: &str) -> bool {
        self.filter.check(header)
    }
}

pub trait RouteTraits<State>: Service<State> + Filter {}
impl<State, T> RouteTraits<State> for T where T: Service<State> + Filter {}

#[derive(Default)]
pub struct Router<State> {
    routes: Vec<Box<dyn RouteTraits<State>>>,
    layers: Vec<Box<dyn Layer>>,
}

impl<State: Send + Sync + 'static> Router<State> {
    pub fn route(mut self, filter: impl Filter, service: impl Service<State>) -> Self {
        self.routes.push(Box::new(Route { filter, service }));
        self
    }

    pub fn layer(mut self, layer: impl Layer) -> Self {
        self.layers.push(Box::new(layer));
        self
    }
}

#[async_trait]
impl<State: Sync + Send + 'static> Service<State> for Router<State> {
    async fn call(&self, request: Request, state: State) -> ResponseResult {
        let request = self
            .layers
            .iter()
            .fold(request, |request, layer| layer.map_request(request));

        let route = self
            .routes
            .iter()
            .find(|route| route.check(&request.header));

        let response = match route {
            Some(route) => route.clone().call(request, state).await,
            None => Response::none(),
        };

        self.layers
            .iter()
            .fold(response, |response, layer| layer.map_response(response))
    }
}

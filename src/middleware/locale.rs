use actix_web::{
    self,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderValue,
    HttpMessage,
};
use fluent::{bundle::FluentBundle, FluentResource};
use futures_util::future::LocalBoxFuture;
use intl_memoizer::concurrent::IntlLangMemoizer;
use log::warn;
use unic_langid::{langid, LanguageIdentifier};

use std::collections::BTreeMap;
use std::fs::{read_dir, File};
use std::future::{ready, Ready};
use std::io::Read;
use std::sync::Arc;

use crate::Error;

#[derive(Clone)]
pub struct LocaleFactory {
    language_bundles:
        BTreeMap<LanguageIdentifier, Arc<FluentBundle<FluentResource, IntlLangMemoizer>>>,
}

impl LocaleFactory {
    pub(crate) fn new(locale_dir: &str) -> Result<Self, Error> {
        let mut language_bundles = BTreeMap::new();
        let reading = match read_dir(locale_dir) {
            Ok(r) => r,
            Err(err) => {
                warn!("Could not read directory: {}", err);
                return Err(err.into());
            }
        };
        for entry in reading.filter_map(|e| {
            if e.is_err() {
                warn!("Could not read directory entry in {}", locale_dir);
            }
            e.ok()
        }) {
            let mut path = entry.path();
            let locale_id: LanguageIdentifier = if let Some(Some(Ok(id))) = path
                .file_name()
                .map(|name| name.to_str().map(|str_name| str_name.parse()))
            {
                id
            } else {
                warn!("Directory with unexpected name in {}", locale_dir);
                continue;
            };
            path.push("send.ftl");
            let mut content = String::new();
            match File::open(path).map(|mut file| file.read_to_string(&mut content)) {
                Ok(Ok(_)) => (),
                Ok(Err(err)) => warn!("Could not read from file: {}", err),
                Err(err) => warn!("Could not open file: {}", err),
            }
            match FluentResource::try_new(content) {
                Ok(res) => {
                    let mut bundle = FluentBundle::new_concurrent(vec![locale_id.clone()]);
                    bundle.add_resource(res).unwrap(); // Works, because we only add one resource to each bundle.
                    language_bundles.insert(locale_id, Arc::new(bundle));
                }
                Err(_) => warn!("Could not parse locale resource."),
            }
        }

        // Make sure the default locale is saved into the BTree:
        assert!(language_bundles.contains_key(&langid!("en-US")));

        Ok(Self { language_bundles })
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for LocaleFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = LocaleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LocaleMiddleware {
            service,
            language_bundles: self.language_bundles.clone(),
        }))
    }
}

pub struct LocaleMiddleware<S> {
    service: S,
    language_bundles:
        BTreeMap<LanguageIdentifier, Arc<FluentBundle<FluentResource, IntlLangMemoizer>>>,
}

impl<S, B> Service<ServiceRequest> for LocaleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let locale: LanguageIdentifier = req
            .headers()
            .get("Accept-Language")
            .unwrap_or(&HeaderValue::from_str("en-US").unwrap())
            .to_str()
            .unwrap_or("en-US")
            .parse()
            .unwrap_or(langid!("en-US"));
        req.extensions_mut().insert(
            self.language_bundles
                .get(&locale)
                .unwrap_or_else(|| self.language_bundles.get(&langid!("en-US")).unwrap())
                .clone(),
        );

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

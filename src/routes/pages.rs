use actix_web::{error, get, web, HttpResponse, Responder};
use fluent::{bundle::FluentBundle, FluentArgs, FluentError, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use log::error;
use tera::Tera;

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::Config;

struct TextMapBuilder<'a> {
    bundle: Arc<FluentBundle<FluentResource, IntlLangMemoizer>>,
    fmt_errors: Vec<FluentError>,
    text_ids: Vec<(&'a str, Option<&'a FluentArgs<'a>>)>,
}

impl<'a, 'b> TextMapBuilder<'a> {
    fn add_text(&mut self, text_id: &'a str, args: Option<&'a FluentArgs<'a>>) {
        self.text_ids.push((text_id, args));
    }

    fn build(&'a mut self) -> HashMap<&'a str, Cow<'a, str>> {
        let mut res = HashMap::new();
        for (text_id, args) in self.text_ids.iter() {
            res.insert(
                *text_id,
                self.bundle.format_pattern(
                    self.bundle
                        .get_message(text_id)
                        .expect("Missing text in Fluent Bundle")
                        .value()
                        .expect("Message without value"),
                    *args,
                    &mut self.fmt_errors,
                ),
            );
        }

        res
    }
}

impl<'a> From<Arc<FluentBundle<FluentResource, IntlLangMemoizer>>> for TextMapBuilder<'a> {
    fn from(bundle: Arc<FluentBundle<FluentResource, IntlLangMemoizer>>) -> Self {
        TextMapBuilder {
            bundle,
            fmt_errors: vec![],
            text_ids: Vec::new(),
        }
    }
}

#[get("/")]
async fn index(
    config: web::Data<Arc<Config>>,
    tmpl: web::Data<Tera>,
    text_bundle: web::ReqData<Arc<FluentBundle<FluentResource, IntlLangMemoizer>>>,
    path_map: web::Data<Arc<HashMap<&str, String>>>,
) -> impl Responder {
    let config = config.into_inner();
    let path_map = path_map.into_inner();

    let mut context = tera::Context::new();

    // Context for index.html.tera:
    context.insert("LOCALE", &format!("{}", text_bundle.locales[0]));

    // Insert PATH map:
    let type_ref: &HashMap<_, _> = path_map.as_ref();
    context.insert("PATHS", type_ref);
    context.insert("BASEURL", &format!("http://{}", config.servername));

    // Context for head.html.tera:
    context.insert("TITLE", "Rust-Send");

    // Context for footer.html.tera:
    context.insert("DONATE_URL", &false); // TODO: Get value from config.
    context.insert("CLI_URL", &false); // TODO: Get value from config.
    context.insert("DMCA_URL", &false); // TODO: Get value from config.
    context.insert("SOURCE_URL", &false); // TODO: Get value from config.

    // Context for initScript.js.tera:
    let empty_map: HashMap<String, String> = HashMap::new();
    context.insert("LIMITS", &empty_map);
    context.insert("WEB_UI", &empty_map);
    context.insert("DEFAULTS", &empty_map);

    // Insert text/language content (TEXTS):
    let mut text_builder = TextMapBuilder::from(text_bundle.into_inner());
    // noscript.html.tera:
    text_builder.add_text("javascriptRequired", None);
    text_builder.add_text("whyJavascript", None);
    text_builder.add_text("enableJavascript", None);
    // header.html.tera:
    text_builder.add_text("title", None);
    // home.html.tera:
    text_builder.add_text("dragAndDropFiles", None);
    let mut args = FluentArgs::new();
    args.set("size", "1111"); // TODO: Set to MAX_SIZE value from config.
    text_builder.add_text("orClickWithSize", Some(&args));
    text_builder.add_text("addFilesButton", None);
    text_builder.add_text("introTitle", None);
    text_builder.add_text("introDescription", None);
    // footer.html.tera:
    text_builder.add_text("footerText", None);
    text_builder.add_text("footerLinkDonate", None);
    text_builder.add_text("footerLinkCli", None);
    text_builder.add_text("footerLinkDmca", None);
    text_builder.add_text("footerLinkSource", None);

    let text_map = text_builder.build();
    context.insert("TEXTS", &text_map);

    tmpl.render("index.html.tera", &context)
        .map(|page| HttpResponse::Ok().body(page))
        .map_err(|err| {
            error!("Could not render template: {}", err);
            error::ErrorInternalServerError("Could not render template.")
        })
}

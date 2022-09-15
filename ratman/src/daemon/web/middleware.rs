#[derive(Default)]
pub(super) struct Instrument {
    metrics: metrics::Metrics,
}

impl Instrument {
    pub fn register_metrics(&self, registry: &mut prometheus_client::registry::Registry) {
        self.metrics.register(registry);
    }
}

#[tide::utils::async_trait]
impl tide::Middleware<super::State> for Instrument {
    async fn handle(
        &self,
        req: tide::Request<super::State>,
        next: tide::Next<'_, super::State>,
    ) -> tide::Result {
        let path = req.url().path().into();
        let method = metrics::HTTPMethod(req.method());
        let rsp = next.run(req).await;

        self.metrics
            .http_requests_total
            .get_or_create(&metrics::HTTPLabels {
                path,
                method,
                status: metrics::HTTPStatusCode(rsp.status()),
            })
            .inc();
        Ok(rsp)
    }
}

mod metrics {
    use prometheus_client::{
        encoding::text::Encode,
        metrics::{counter::Counter, family::Family},
    };

    #[derive(Default)]
    pub(super) struct Metrics {
        pub http_requests_total: Family<HTTPLabels, Counter>,
    }

    impl Metrics {
        pub fn register(&self, registry: &mut prometheus_client::registry::Registry) {
            registry.register(
                "http_requests",
                "Total number of HTTP requests",
                Box::new(self.http_requests_total.clone()),
            );
        }
    }

    #[derive(Clone, Hash, PartialEq, Eq, Encode)]
    pub(super) struct HTTPLabels {
        pub method: HTTPMethod,
        pub path: String,
        pub status: HTTPStatusCode,
    }

    /// Encode'able wrapper for tide::http::Method.
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(super) struct HTTPMethod(pub tide::http::Method);

    impl Encode for HTTPMethod {
        fn encode(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
            write!(writer, "{}", self.0)
        }
    }

    /// Encode'able wrapper for tide::http::Method.
    #[derive(Clone, Hash, PartialEq, Eq)]
    pub(super) struct HTTPStatusCode(pub tide::http::StatusCode);

    impl Encode for HTTPStatusCode {
        fn encode(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
            write!(writer, "{}", self.0)
        }
    }
}

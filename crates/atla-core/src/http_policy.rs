use std::time::Duration;

/// Network limits applied to every Atlassian HTTP client.
///
/// API calls use a bounded request timeout so an automated caller cannot hang
/// indefinitely. Binary transfers and multipart uploads get a larger budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HttpPolicy {
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
    pub transfer_timeout: Duration,
}

impl Default for HttpPolicy {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(60),
            transfer_timeout: Duration::from_secs(300),
        }
    }
}

impl HttpPolicy {
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = self.connect_timeout.min(timeout);
        self.request_timeout = timeout;
        self.transfer_timeout = timeout;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}

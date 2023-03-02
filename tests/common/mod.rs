use httpmock::{Method::POST, Mock, MockServer, Regex};

struct RikaFirenetMock<'mock> {
    mock_base_url: String,
    login: Mock<'mock>,
}

impl<'mock> RikaFirenetMock<'mock> {
    fn start() -> RikaFirenetMock<'mock> {
        let server = MockServer::start();
        RikaFirenetMock {
            mock_base_url: server.base_url(),
            login: 
        }
    }
}

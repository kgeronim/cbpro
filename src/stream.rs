use core::{
    future::Future, 
    pin::Pin
};
use futures::{
    stream::Stream,
    task::{Context, Poll},
};
use reqwest::{Client, Error, Response, Url};
use serde_json::Value;
use futures::stream::{StreamExt};

enum State {
    Start,
    Stop,
}

pub(super) struct Paginate {
    in_flight: ResponseFuture,
    client: Client,
    url: Url,
    limit: String,
    state: State,
}

impl Paginate {
    pub(super) fn new(
        client: Client,
        url: Url,
        limit: String
    ) -> Self {
        Self {
            in_flight: ResponseFuture::new(Box::new(client.get(url.clone()).send())),
            client,
            url,
            limit,
            state: State::Start,
        }
    }

    pub(super) fn into_json(self) -> JsonStream {
        JsonStream::new(Box::new(self.then(|x| async move { x?.json::<Value>().await })))
    }

    fn in_flight(self: Pin<&mut Self>) -> Pin<&mut ResponseFuture> {
        unsafe { Pin::map_unchecked_mut(self, |x| &mut x.in_flight) }
    }
}

impl Stream for Paginate {
    type Item = Result<Response, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let State::Stop = self.state {
            return Poll::Ready(None);
        }

        let res = match self.as_mut().in_flight().as_mut().poll(cx) {
            Poll::Ready(Err(e)) => {
                return Poll::Ready(Some(Err(e)));
            }
            Poll::Ready(Ok(res)) => res,
            Poll::Pending => return Poll::Pending,
        };

        if let Some(after) = res.headers().get("cb-after") {
            let after = String::from(after.to_str().unwrap());
            self.in_flight = ResponseFuture::new(Box::new(
                self.client.get(self.url.clone()).query(&[("limit", &self.limit), ("after", &after)]).send(),
            ));
        } else {
            self.state = State::Stop;
        }
        Poll::Ready(Some(Ok(res)))
    }
}

struct ResponseFuture {
    inner: Pin<Box<dyn Future<Output = Result<Response, Error>> + Send>>,
}

impl ResponseFuture {
    fn new(fut: Box<dyn Future<Output = Result<Response, Error>> + Send>) -> Self {
        Self { inner: fut.into() }
    }
}

impl Future for ResponseFuture {
    type Output = Result<Response, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

pub struct JsonStream {
    inner: Pin<Box<dyn Stream<Item = Result<Value, Error>> + Send>>,
}

impl JsonStream {
    pub fn new(stream: Box<dyn Stream<Item = Result<Value, Error>> + Send>) -> Self {
        Self { inner: stream.into() }
    }
}

impl Stream for JsonStream {
    type Item = Result<Value, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx)
    }
}

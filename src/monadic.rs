use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct ParseError<S> {
    pub state: S,
    pub msg: String,
}

pub type Matcher<S, T> = dyn Fn(S) -> Result<(T, S), ParseError<S>> + 'static;

pub struct Parser<S, T> {
    pub f: Box<Matcher<S, T>>,
}

impl<S, T> Parser<S, T>
where
    T: 'static,
    S: Clone + 'static,
{
    pub fn unit(f: Box<Matcher<S, T>>) -> Parser<S, T> {
        Parser { f }
    }

    pub fn map<U: 'static, F: Fn(T) -> U + 'static>(self, f: F) -> Parser<S, U> {
        Parser::unit(Box::new(move |state| {
            let (x, state_) = (self.f)(state)?;
            Ok((f(x), state_))
        }))
    }

    pub fn pure<F: Fn() -> T + 'static>(f: F) -> Parser<S, T> {
        Parser::unit(Box::new(move |state| Ok((f(), state))))
    }

    pub fn apply<U: 'static, F: Fn(T) -> U + 'static>(self, lifted: Parser<S, F>) -> Parser<S, U> {
        Parser::unit(Box::new(move |state| {
            let (f, state_) = (lifted.f)(state)?;
            let (x, state__) = (self.f)(state_)?;
            Ok((f(x), state__))
        }))
    }

    pub fn bind<U: 'static, F: Fn(T) -> Parser<S, U> + 'static>(self, f: F) -> Parser<S, U> {
        Parser::unit(Box::new(move |state| {
            let (x, state_) = (self.f)(state)?;
            let (y, state__) = (f(x).f)(state_)?;
            Ok((y, state__))
        }))
    }

    pub fn failure<F: Fn() -> String + 'static>(f: F) -> Parser<S, T> {
        Parser::unit(Box::new(move |state| Err(ParseError { state, msg: f() })))
    }

    pub fn or(self, that: Parser<S, T>) -> Parser<S, T> {
        Parser::unit(Box::new(move |state| match (self.f)(state.clone()) {
            Err(_) => (that.f)(state),
            Ok(ret) => Ok(ret),
        }))
    }

    pub fn many(self) -> Parser<S, Vec<T>> {
        Parser::unit(Box::new(move |state| {
            let mut acc = vec![];
            let mut state = state;
            loop {
                match (self.f)(state.clone()) {
                    Err(_) => {
                        return Ok((acc, state));
                    }
                    Ok((value, state_)) => {
                        state = state_;
                        acc.push(value);
                    }
                }
            }
        }))
    }

    pub fn chain<U: 'static>(self, next: Parser<S, U>) -> Parser<S, (T, U)> {
        Parser::unit(Box::new(move |state| {
            let (x, state_) = (self.f)(state)?;
            let (y, state__) = (next.f)(state_)?;
            Ok(((x, y), state__))
        }))
    }
}

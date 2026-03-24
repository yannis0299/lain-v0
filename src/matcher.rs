use eyre::{bail, Result, WrapErr};

pub type Matcher<S, T> = dyn Fn(&mut S) -> Result<T> + 'static;

pub struct MonadMatcher<S, T>(pub Box<Matcher<S, T>>);

impl<S, T> MonadMatcher<S, T>
where
    T: 'static,
    S: 'static + Clone,
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut S) -> Result<T> + 'static,
    {
        MonadMatcher(Box::new(f))
    }

    pub fn pure(ret: T) -> Self
    where
        T: Clone,
    {
        MonadMatcher::new(move |_| Ok(ret.clone()))
    }

    pub fn failure<F>(f: F) -> Self
    where
        F: Fn() -> Result<T> + 'static,
    {
        MonadMatcher::new(move |_| f())
    }

    pub fn or(self, that: Self) -> Self {
        MonadMatcher::new(move |state: &mut S| {
            let checkpoint = state.clone();
            match (self.0)(state) {
                Err(_) => {
                    *state = checkpoint;
                    (that.0)(state)
                }
                Ok(ret) => Ok(ret),
            }
        })
    }

    pub fn optional(self) -> MonadMatcher<S, Option<T>> {
        MonadMatcher::new(move |state: &mut S| {
            let checkpoint = state.clone();
            match (self.0)(state) {
                Err(_) => {
                    *state = checkpoint;
                    Ok(None)
                }
                Ok(ret) => Ok(Some(ret)),
            }
        })
    }

    pub fn fold(matchers: Vec<Self>) -> Self {
        let mut iter = matchers.into_iter();
        let mut acc = iter
            .next()
            .expect("MonadMatcher: fold requires at least one matcher");
        for matcher in iter {
            acc = acc.or(matcher);
        }
        acc
    }

    pub fn map<U, F>(self, f: F) -> MonadMatcher<S, U>
    where
        U: 'static,
        F: Fn(T) -> U + 'static,
    {
        MonadMatcher::new(move |state| {
            let x = (self.0)(state)?;
            Ok(f(x))
        })
    }

    pub fn then<U, F>(self, f: F) -> MonadMatcher<S, U>
    where
        U: 'static,
        F: Fn(T) -> MonadMatcher<S, U> + 'static,
    {
        MonadMatcher::new(move |state| {
            let x = (self.0)(state)?;
            let y = ((f(x)).0)(state)?;
            Ok(y)
        })
    }

    pub fn chain<U>(self, that: MonadMatcher<S, U>) -> MonadMatcher<S, (T, U)>
    where
        U: 'static,
    {
        MonadMatcher::new(move |state: &mut S| {
            let x = (self.0)(state)?;
            let y = (that.0)(state)?;
            Ok((x, y))
        })
    }

    pub fn many(self) -> MonadMatcher<S, Vec<T>> {
        MonadMatcher::new(move |state: &mut S| {
            let mut acc = vec![];
            loop {
                let checkpoint = state.clone();
                match (self.0)(state) {
                    Ok(ret) => acc.push(ret),
                    Err(_) => {
                        *state = checkpoint;
                        break;
                    }
                }
            }
            Ok(acc)
        })
    }

    pub fn many1(self) -> MonadMatcher<S, Vec<T>> {
        MonadMatcher::new(move |state: &mut S| {
            let mut acc = vec![];
            loop {
                let checkpoint = state.clone();
                match (self.0)(state) {
                    Ok(ret) => acc.push(ret),
                    Err(_) => {
                        *state = checkpoint;
                        break;
                    }
                }
            }
            if acc.is_empty() {
                bail!("MonadMatcher: many1 expects at least one match!")
            }
            Ok(acc)
        })
    }
}

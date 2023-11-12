use itertools::{Either, Itertools as _};

pub trait IterExt<T>: Iterator<Item = Result<T, syn::Error>> {
    fn collect_fallible<B>(self) -> Result<B, syn::Error>
    where
        B: Default + Extend<T>;
}

impl<I, T> IterExt<T> for I
where
    I: Iterator<Item = Result<T, syn::Error>>,
{
    fn collect_fallible<B>(self) -> Result<B, syn::Error>
    where
        B: Default + Extend<T>,
    {
        let (good_data, errors): (B, Vec<_>) = self.partition_map(|r| match r {
            Ok(v) => Either::Left(v),
            Err(v) => Either::Right(v),
        });

        if let Some(e) = errors.into_iter().reduce(|mut l, r| {
            l.combine(r);
            l
        }) {
            Err(e)
        } else {
            Ok(good_data)
        }
    }
}

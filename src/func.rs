// could be useful some day

impl <X, R, E> IsApi for fn(X) -> Result<R, E> {
  type MethodList = build_hlist!(X);
    const DESCRIPTION: &str = "Anonimous function";
    const NAME: &str = "fn";
}

impl <X, R, E> ApiMethod<fn(X) -> Result<R, E>> for fn(X) -> Result<R, E> {
  type Res = R;
    const DESCRIPTION: &str = "Anonimous function";
    const NAME: &str = "fn";
}

impl <X, R, E> ImplsApi<fn(X) -> Result<R, E>> for fn(X) -> Result<R, E> {
  type Err = E;
}

impl<X, R, E> ImplsMethod<fn(X) -> Result<R, E>, X>
    for fn(X) -> Result<R, E>
    where X: ApiMethod<Self, Res = R> + Send
{
    async fn call_api(&self, req: X) -> Result<R, E> {
        self(req)
    }
}

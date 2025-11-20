#![allow(unused_macros, unused_imports, dead_code)]
use crate::{HasMethod, ImplsMethod, IsApi};
use std::pin::Pin;

macro_rules! hvtable {
  () => { () };
  ($x:expr $(, $rest:expr)*) => { (Box::new($x), hvtable!($($rest),*)) };
}

trait BuildHVTable<API> {
  type HVTable;
}

impl<API, H, T> BuildHVTable<API> for (H, T)
where
  API: IsApi + HasMethod<H>,
  T: BuildHVTable<API>,
{
  type HVTable = (Box<dyn Fn(H) -> API::Res + Send + Sync>, T::HVTable);
}

// Clean but requires core traits modifications:
trait CallHVTable<API: HasMethod<M>, M, I> {
  fn call_vtable_api(&self, req: M) -> API::Res;
}

impl<API, M, T> CallHVTable<API, M, ()> for (Box<dyn Fn(M) -> API::Res + Send + Sync>, T)
where
  API: HasMethod<M>,
  //H: Fn(M) -> API::Res,
{
  fn call_vtable_api(&self, req: M) -> API::Res {
    self.0(req)
  }
}

impl<API, M, I, H, T> CallHVTable<API, M, (I,)> for (H, T)
where
  API: HasMethod<M>,
  T: CallHVTable<API, M, I>,
{
  fn call_vtable_api(&self, req: M) -> API::Res {
    self.1.call_vtable_api(req)
  }
}

struct VTable<HL>(pub HL);

impl<API> BuildHVTable<API> for () {
  type HVTable = ();
}

impl<API, M, HL> ImplsMethod<API, M> for VTable<HL>
where
  API: IsApi + HasMethod<M> + HasMethodExt<M>,
  HL: CallHVTable<API, M, <<API::Methods as HLen>::Len as Subtract<(API::MethodIndex,)>>::Difference>
    + Sync,
  API::Methods: HLen,
  <API::Methods as HLen>::Len: Subtract<(API::MethodIndex,)>,
  M: Send,
{
  async fn call_api(&self, req: M) -> API::Res {
    self.0.call_vtable_api(req)
  }
}

pub trait Subtract<B> {
  type Difference;
}
impl<A> Subtract<()> for A {
  type Difference = A;
}
impl<A: Subtract<B>, B> Subtract<(B,)> for (A,) {
  type Difference = A::Difference;
}

pub trait HLen {
  type Len;
}
impl<H, T: HLen> HLen for (H, T) {
  type Len = (T::Len,);
}
impl HLen for () {
  type Len = ();
}

/// Trait alias for `<API::Methods as BuildHVTable<API>>::HVTable`
trait HVTable {
  type HVTable;
}
impl<API> HVTable for API
where
  API: IsApi,
  API::Methods: BuildHVTable<API>,
{
  type HVTable = <API::Methods as BuildHVTable<API>>::HVTable;
}

pub trait HasMethodExt<M>: HasMethod<M> {
  type MethodIndex;
}

macro_rules! mk_index { // mk peano number
  () => { () };
  ($x:ty $(, $rest:ty)*) => { (mk_index!($($rest),*),) };
}

#[cfg(test)]
mod test {
  use super::*;

  struct SomeAPI;
  crate::define_api! { SomeAPI => {
    "get_a", GetA => bool;
    "post_a", PostA => Result<(), String>;
  } }

  #[derive(Clone)]
  pub struct GetA;
  #[derive(Clone)]
  pub struct PostA(pub bool);

  // would need to be included in `define_api`
  macro_rules! impl_has_method_ext_for_SomeAPI {
    () => {};
    ($m:ty $(, $rest:ty)*) => {
        impl HasMethodExt<$m> for SomeAPI {
          type MethodIndex = mk_index!($($rest),*);
        }
        impl_has_method_ext_for_SomeAPI!{$($rest),*}
    }
  }
  per_SomeAPI_method! {impl_has_method_ext_for_SomeAPI}

  // helper trait to assert type equality
  trait TypeEq<A, B> {
    const YES: bool = true;
  }
  impl<A> TypeEq<A, A> for () {}

  #[tokio::test]
  async fn adsf() {
    <() as TypeEq<<SomeAPI as HasMethodExt<GetA>>::MethodIndex, ((),)>>::YES;
    <() as TypeEq<<SomeAPI as HasMethodExt<PostA>>::MethodIndex, ()>>::YES;

    type Len = <<SomeAPI as IsApi>::Methods as HLen>::Len;
    type GetAidx = <SomeAPI as HasMethodExt<GetA>>::MethodIndex;
    <() as TypeEq<Len, (((),),)>>::YES;
    <() as TypeEq<<Len as Subtract<GetAidx>>::Difference, ((),)>>::YES;

    use crate::CallApi;
    let backend: VTable<<SomeAPI as HVTable>::HVTable> =
      VTable(hvtable!(|GetA| true, |PostA(_)| Ok(())));

    // works:
    assert!(backend.call_api_x::<SomeAPI, _>(GetA).await);
    assert!(backend.call_api_x::<SomeAPI, _>(PostA(false)).await.is_ok());

    // doesn't work (cannot infer type of the API):
    // assert!(backend.call_api(GetA).await);
    /*
    assert!(
      <VTable<(
        Box<dyn Fn(GetA) -> bool + Send + Sync>,
        (Box<dyn Fn(PostA) -> Result<(), std::string::String> + Send + Sync>, ())
      )> as ImplsMethod<_, GetA>>::call_api(&backend, GetA)
      .await
    );
    */

    //assert!(backend.call_api(PostA(false)).await.is_ok());
  }
}

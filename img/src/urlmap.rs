#[macro_export]
macro_rules! urlmap {
  ($router: expr) => {
    $router.route("/:args/:id", get(root))
  };
}

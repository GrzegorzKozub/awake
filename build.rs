fn main() {
  embed_resource::compile("assets/awake.rc", embed_resource::NONE)
    .manifest_required()
    .unwrap();
}

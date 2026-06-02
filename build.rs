fn main() {
    // Tambah embed_resource::NONE sebagai argumen kedua
    // Tambah .manifest_optional().unwrap() karena return type adalah #[must_use]
    embed_resource::compile("app.rc", embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}
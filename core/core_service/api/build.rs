fn main() {
    let _user_proto_file = "./proto/user_services.proto";

    tonic_build::configure()
        .build_server(true)
        .compile(&[_user_proto_file], &["."])
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", _user_proto_file);
}

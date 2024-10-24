fn main() {
    let _user_proto_file = "./proto/user_services.proto";
    let _projects_proto_file = "./proto/projects_services.proto";
    let _default_service_file = "./proto/default_service.proto";

    tonic_build::configure()
        .build_server(true)
        .compile_protos(
            &[_user_proto_file, _user_proto_file, _projects_proto_file],
            &["."],
        )
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!(
        "cargo:rerun-if-changed={} {}",
        _user_proto_file, _user_proto_file
    );
}

fn main() {
    let _user_proto_file = "./proto/user_services.proto";
    let _projects_proto_file = "./proto/projects_services.proto";
    let _default_service_file = "./proto/default_service.proto";
    let _companies_services_file = "./proto/companies_services.proto";
    let _notes_services_proto_file = "./proto/notes_services.proto";
    let _tasks_services_proto_file = "./proto/tasks_services.proto";

    tonic_build::configure()
        .build_server(true)
        .compile_protos(
            &[
                _user_proto_file,
                _projects_proto_file,
                _default_service_file,
                _companies_services_file,
                _notes_services_proto_file,
                _tasks_services_proto_file,
            ],
            &["."],
        )
        .unwrap_or_else(|e| panic!("protobuf compile error: {}", e));

    println!(
        "cargo:rerun-if-changed={} {} {} {} {} {}",
        _user_proto_file,
        _projects_proto_file,
        _default_service_file,
        _companies_services_file,
        _notes_services_proto_file,
        _tasks_services_proto_file,
    );
}

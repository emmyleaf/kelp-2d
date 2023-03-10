#[cfg(test)]
pub mod csharp {
    use super::super::*;
    use interoptopus::{function, util::NamespaceMappings, Interop, Inventory, InventoryBuilder};
    use interoptopus_backend_csharp::{
        overloads::DotNet, CSharpVisibility::ForceInternal, Config, Generator, Unsafe::UnsafePlatformMemCpy,
    };

    pub fn ffi_inventory() -> Inventory {
        InventoryBuilder::new()
            .register(function!(initialise))
            .register(function!(render_pass))
            .register(function!(create_texture_with_data))
            // .register(function!(free_texture))
            .register(function!(set_surface_size))
            .register(function!(uninitialise))
            .inventory()
    }

    #[test]
    pub fn csharp() {
        let config = Config {
            class: "Native".to_string(),
            dll_name: "kelp-2d".to_string(),
            namespace_mappings: NamespaceMappings::new("Kelp2d"),
            visibility_types: ForceInternal,
            use_unsafe: UnsafePlatformMemCpy,
            rename_symbols: true,
            ..Config::default()
        };

        Generator::new(config, ffi_inventory())
            .add_overload_writer(DotNet::new())
            .write_file("bindings/Native.g.cs")
            .unwrap();
    }
}

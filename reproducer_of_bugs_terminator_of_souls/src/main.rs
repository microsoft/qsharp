fn main() {
    let packages = vec![
        (
            "PackageA",
            r"
                    export Microsoft.Quantum.Core.Length as Foo;
                ",
        ),
        (
            "PackageB",
            r"

                    import PackageA.PackageA.Foo;
                    @EntryPoint()
                    function Main() : Unit {
                        let len = Foo([1, 2]);
                    }
                ",
        ),
    ];

    compile_with_frontend(packages.clone());
    compile_with_qsc_top_level(packages.clone());
}

fn compile_with_frontend(packages: Vec<(&str, &str)>) {
    use std::sync::Arc;

    use qsc::{hir::PackageId, PackageStore, SourceMap, TargetCapabilityFlags};
    use qsc_frontend::compile;
    let last_source = packages.last().unwrap();
    let mut store = PackageStore::new(qsc::compile::core());
    let mut prev_id_and_name: Option<(PackageId, &str)> = None;
    let num_packages = packages.len();
    for (package_name, package_source) in packages.clone().into_iter().take(num_packages - 1) {
        let deps = if let Some((prev_id, prev_name)) = prev_id_and_name {
            vec![(prev_id, Some(Arc::from(prev_name)))]
        } else {
            vec![]
        };

        let sources = SourceMap::new(
            [(
                Arc::from(format!("{package_name}.qs")),
                Arc::from(package_source),
            )],
            None,
        );

        let unit = compile::compile(
            &store,
            &deps[..],
            sources,
            TargetCapabilityFlags::all(),
            qsc::LanguageFeatures::default(),
        );

        prev_id_and_name = Some((store.insert(unit), package_name));
    }

    println!("constructing incremental compiler...");
    let deps = if let Some((prev_id, prev_name)) = prev_id_and_name {
        vec![(prev_id, Some(Arc::from(prev_name)))]
    } else {
        vec![]
    };
    let mut compiler = qsc_frontend::incremental::Compiler::new(
        &store,
        &deps[..],
        TargetCapabilityFlags::all(),
        qsc::LanguageFeatures::default(),
    );

    let e = compiler
        .compile_fragments(
            &mut store.get((num_packages - 2).into()).unwrap().clone(),
            last_source.0.clone(),
            last_source.1.clone(),
            |errs| {
                if !errs.is_empty() {
                    println!("Errs: {errs:?}");
                    panic!()
                } else {
                    Result::<(), ()>::Ok(())
                }
            },
        )
        .unwrap();

    println!("Code passed with incremental compiler from qsc_frontend.");
}

fn compile_with_qsc_top_level(packages: Vec<(&str, &str)>) {
    use std::sync::Arc;

    use qsc::compile;
    use qsc::PackageType;
    use qsc::{hir::PackageId, PackageStore, SourceMap, TargetCapabilityFlags};
    let last_source = packages.last().unwrap().clone();
    let mut store = PackageStore::new(qsc::compile::core());
    let mut prev_id_and_name: Option<(PackageId, &str)> = None;
    let num_packages = packages.len();
    for (package_name, package_source) in packages.clone().into_iter().take(num_packages - 1) {
        let deps = if let Some((prev_id, prev_name)) = prev_id_and_name {
            vec![(prev_id, Some(Arc::from(prev_name)))]
        } else {
            vec![]
        };

        let sources = SourceMap::new(
            [(
                Arc::from(format!("{package_name}.qs")),
                Arc::from(package_source),
            )],
            None,
        );

        let (unit, errs) = compile::compile(
            &store,
            &deps[..],
            sources,
            PackageType::Lib,
            TargetCapabilityFlags::all(),
            qsc::LanguageFeatures::default(),
        );
        if !errs.is_empty() {
            println!("Errs: {errs:?}");
            panic!()
        }

        prev_id_and_name = Some((store.insert(unit), package_name));
    }

    println!("constructing incremental compiler...");
    let deps = if let Some((prev_id, prev_name)) = prev_id_and_name {
        vec![(prev_id, Some(Arc::from(prev_name)))]
    } else {
        vec![]
    };

    let last_source_as_map = SourceMap::new(
        [(
            Arc::from(last_source.0.to_string()),
            Arc::from(last_source.1),
        )],
        None,
    );

    let mut compiler = qsc::incremental::Compiler::new(
        last_source_as_map,
        PackageType::Exe,
        TargetCapabilityFlags::all(),
        qsc::LanguageFeatures::default(),
        store,
        &deps[..],
    )
    .unwrap();

    let e = compiler
        .compile_fragments(last_source.0.clone(), last_source.1.clone(), |errs| {
            if !errs.is_empty() {
                println!("Errs: {errs:?}");
                panic!()
            } else {
                Result::<(), _>::Ok(())
            }
        })
        .unwrap();

    println!("Code passed with incremental compiler from qsc.");
}

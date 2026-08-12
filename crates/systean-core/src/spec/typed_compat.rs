use crate::semantics::{
    Environment, EnvironmentError, FunctionParameter, Origin, Parameter, Signature, Type,
};

use super::{CompiledSymbolKind, CompiledType, TypedSemanticPackage};

/// Phase 18 compatibility projection for the Phase 17 syntax/checker runtime.
///
/// Semantic ownership remains in `TypedSemanticPackage`; this environment is derived from stable
/// typed IDs and source/debug labels only. Phase 19 removes this projection when surface grammar
/// consumes typed symbols directly.
pub(crate) fn project_environment(
    package: &TypedSemanticPackage,
) -> Result<Environment, EnvironmentError> {
    let mut environment = Environment::new();

    for (id, arity) in package.types() {
        let name = package
            .source_name_for_type(id)
            .expect("compiled type must retain a source/debug name")
            .to_owned();
        let parameters = (0..arity).map(|index| format!("T{index}")).collect();
        environment.define_type(name.clone(), parameters)?;
        if let Some(provenance) = package.type_provenance(id) {
            environment.set_type_origin(
                name,
                Origin::new(provenance.source.clone(), provenance.declaration),
            );
        }
    }

    for (child, parent) in package.subtypes() {
        let child_name = package
            .source_name_for_type(child)
            .expect("compiled subtype child must have a debug name")
            .to_owned();
        let parent_name = package
            .source_name_for_type(parent)
            .expect("compiled subtype parent must have a debug name")
            .to_owned();
        environment.define_subtype(child_name, parent_name)?;
    }

    for (kind, ty) in package.literal_types() {
        environment.define_literal_type(kind, legacy_type(package, ty))?;
        if let Some(provenance) = package.literal_provenance(kind) {
            environment.set_literal_origin(
                kind,
                Origin::new(provenance.source.clone(), provenance.declaration),
            );
        }
    }

    for symbol in package.symbols() {
        let name = package
            .source_name_for_symbol(symbol.id)
            .expect("compiled symbol must retain a source/debug name")
            .to_owned();
        let debug = package
            .debug_symbol(symbol.id)
            .expect("compiled symbol must retain parameter debug labels");
        let origin = Origin::new(
            symbol.provenance.source.clone(),
            symbol.provenance.declaration,
        );

        let is_plain_word_constant = symbol.kind == CompiledSymbolKind::Word
            && symbol.signature.type_parameter_count == 0
            && symbol.signature.parameters.is_empty()
            && symbol.definition.is_none();
        if is_plain_word_constant {
            environment.define_constant(name.clone(), legacy_type(package, &symbol.signature.returns))?;
            environment.set_constant_origin(name, origin);
            continue;
        }

        let signature = Signature {
            type_parameters: (0..symbol.signature.type_parameter_count)
                .map(|index| format!("T{index}"))
                .collect(),
            parameters: symbol
                .signature
                .parameters
                .iter()
                .enumerate()
                .map(|(index, ty)| Parameter {
                    name: debug
                        .parameter_names
                        .get(index)
                        .cloned()
                        .unwrap_or_else(|| format!("p{index}")),
                    ty: legacy_type(package, ty),
                })
                .collect(),
            returns: legacy_type(package, &symbol.signature.returns),
        };
        environment.define_operator(name.clone(), signature)?;
        environment.set_operator_origin(name, origin);
    }

    Ok(environment)
}

pub(crate) fn legacy_type(package: &TypedSemanticPackage, ty: &CompiledType) -> Type {
    match ty {
        CompiledType::Named(id) => Type::Named(
            package
                .source_name_for_type(*id)
                .expect("compiled type must retain a source/debug name")
                .to_owned(),
        ),
        CompiledType::Generic { constructor, arguments } => Type::Generic {
            name: package
                .source_name_for_type(*constructor)
                .expect("compiled generic type must retain a source/debug name")
                .to_owned(),
            arguments: arguments.iter().map(|ty| legacy_type(package, ty)).collect(),
        },
        CompiledType::Variable(index) => Type::Variable(format!("T{index}")),
        CompiledType::Function { parameters, returns } => Type::Function {
            parameters: parameters
                .iter()
                .map(|ty| FunctionParameter {
                    name: None,
                    ty: legacy_type(package, ty),
                })
                .collect(),
            returns: Box::new(legacy_type(package, returns)),
        },
        CompiledType::Record(fields) => Type::Record(
            fields
                .iter()
                .map(|(field, ty)| (field.to_string(), legacy_type(package, ty)))
                .collect(),
        ),
    }
}

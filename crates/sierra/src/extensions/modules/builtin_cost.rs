use super::gas::GasBuiltinType;
use super::range_check::RangeCheckType;
use crate::define_libfunc_hierarchy;
use crate::extensions::lib_func::{
    BranchSignature, DeferredOutputKind, LibFuncSignature, OutputVarInfo, ParamSignature,
    SierraApChange, SignatureSpecializationContext,
};
use crate::extensions::{
    GenericLibFunc, NamedType, OutputVarReferenceInfo, SignatureBasedConcreteLibFunc,
    SpecializationError,
};
use crate::ids::GenericLibFuncId;
use crate::program::GenericArg;

/// Represents different type of costs.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CostTokenType {
    /// A single Cairo step, or some cost which is equivalent to it.
    Step,
    /// One invocation of the pedersen hash function.
    Pedersen,
}
impl CostTokenType {
    pub fn iter() -> std::slice::Iter<'static, Self> {
        [CostTokenType::Step, CostTokenType::Pedersen].iter()
    }

    /// Returns the name of the token type, in snake_case.
    pub fn name(&self) -> String {
        match self {
            CostTokenType::Step => "step",
            CostTokenType::Pedersen => "pedersen",
        }
        .into()
    }
}

define_libfunc_hierarchy! {
    pub enum BuiltinCostLibFunc {
        BuiltinGetGas(BuiltinCostGetGasLibFunc),
    }, BuiltinCostConcreteLibFunc
}

/// LibFunc for getting gas to be used by a builtin.
// TODO(lior): Remove allow(dead_code) once `token_type` is used.
#[allow(dead_code)]
pub struct BuiltinCostGetGasLibFunc {
    token_type: CostTokenType,
}
impl GenericLibFunc for BuiltinCostGetGasLibFunc {
    type Concrete = BuiltinGetGasConcreteLibFunc;

    fn by_id(id: &GenericLibFuncId) -> Option<Self> {
        for token_type in CostTokenType::iter() {
            if *id == GenericLibFuncId::from_string(&format!("{}_get_gas", token_type.name())) {
                return Some(Self { token_type: *token_type });
            }
        }
        None
    }

    fn specialize(
        &self,
        context: &dyn crate::extensions::lib_func::SpecializationContext,
        args: &[crate::program::GenericArg],
    ) -> Result<Self::Concrete, SpecializationError> {
        Ok(BuiltinGetGasConcreteLibFunc {
            signature: self.specialize_signature(context.upcast(), args)?,
            token_type: self.token_type,
        })
    }

    fn specialize_signature(
        &self,
        context: &dyn SignatureSpecializationContext,
        args: &[GenericArg],
    ) -> Result<LibFuncSignature, SpecializationError> {
        if !args.is_empty() {
            return Err(SpecializationError::WrongNumberOfGenericArgs);
        }

        let gas_builtin_type = context.get_concrete_type(GasBuiltinType::id(), &[])?;
        let range_check_type = context.get_concrete_type(RangeCheckType::id(), &[])?;
        Ok(LibFuncSignature {
            param_signatures: vec![
                ParamSignature::new(range_check_type.clone()),
                ParamSignature::new(gas_builtin_type.clone()),
            ],
            branch_signatures: vec![
                // Success:
                BranchSignature {
                    vars: vec![
                        OutputVarInfo {
                            ty: range_check_type.clone(),
                            ref_info: OutputVarReferenceInfo::Deferred(
                                DeferredOutputKind::AddConst { param_idx: 0 },
                            ),
                        },
                        OutputVarInfo {
                            ty: gas_builtin_type.clone(),
                            ref_info: OutputVarReferenceInfo::Deferred(DeferredOutputKind::Generic),
                        },
                    ],
                    ap_change: SierraApChange::Known(2), // TODO: Check/fix.
                },
                // Failure:
                BranchSignature {
                    vars: vec![
                        OutputVarInfo {
                            ty: range_check_type,
                            ref_info: OutputVarReferenceInfo::Deferred(
                                DeferredOutputKind::AddConst { param_idx: 0 },
                            ),
                        },
                        OutputVarInfo {
                            ty: gas_builtin_type,
                            ref_info: OutputVarReferenceInfo::SameAsParam { param_idx: 1 },
                        },
                    ],
                    ap_change: SierraApChange::Known(3), // TODO: Check/fix.
                },
            ],
            fallthrough: Some(0),
        })
    }
}

pub struct BuiltinGetGasConcreteLibFunc {
    pub signature: LibFuncSignature,
    pub token_type: CostTokenType,
}
impl SignatureBasedConcreteLibFunc for BuiltinGetGasConcreteLibFunc {
    fn signature(&self) -> &LibFuncSignature {
        &self.signature
    }
}
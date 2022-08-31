use crate::differs::reduced::change_type::Change;

use super::{calls::prelude::Index, diff_result::DiffResult, pallet_item::PalletItem};
use frame_metadata::PalletMetadata;
use scale_info::form::PortableForm;

#[derive(Debug, PartialEq, Eq)]
pub struct ReducedPallet {
	/// Index of the pallet
	pub index: Index,

	/// Name of the pallet
	pub name: String,

	/// Vec of all the `PalletItem`
	pub items: Vec<PalletItem>,
	// TODO: no doc ?
}

// TODO: impl Iterator
impl ReducedPallet {
	/// Computes the differences between 2 pallets
	pub fn diff<'meta>(
		pallet_a: Option<&'meta Self>,
		pallet_b: Option<&'meta Self>,
	) -> DiffResult<'meta, ReducedPallet> {
		match (pallet_a, pallet_b) {
			(Some(pa), Some(pb)) => {
				assert_eq!(pa.index, pb.index, "Comparing different indexes does not make much sense");
				if pa.name != pb.name {
					return DiffResult::new(Change::Modified((pa, pb)));
				}

				if pa.items != pb.items {
					return DiffResult::new(Change::Modified((pa, pb)));
				}
			}
			(Some(pa), None) => return DiffResult::new(Change::Removed(pa)),
			(None, Some(pb)) => return DiffResult::new(Change::Added(pb)),
			(None, None) => todo!(),
		}

		DiffResult::new(Change::Unchanged)
	}
}

impl From<&PalletMetadata<PortableForm>> for ReducedPallet {
	fn from(pallet: &PalletMetadata<PortableForm>) -> Self {
		let index: Index = pallet.index.into();
		let name = pallet.name.to_string();
		// let items: Vec<PalletItem> = Vec::new();
		// todo!("You are here :)");

		// let calls: Vec<PalletItem> = pallet.calls.as_ref().map(|call| call.into()).unwrap();
		// let registry = pallet.into()

		// not the rigth approach
		// let pallet_meta : PalletMetadata<MetaForm> = pallet.into();

		// let events: Vec<PalletItem> = pallet.event.as_ref().map(|call| call.into()).unwrap();
		// let errors: Vec<PalletItem> = pallet.error.as_ref().map(|call| call.into()).unwrap();
		// let storages: Vec<PalletItem> = pallet.storage.as_ref().map(|call| call.into()).unwrap();
		// let constants: Vec<PalletItem> = pallet.constants.as_ref().map(|call| call.into()).unwrap();
		// TODO: Add others as well
		// let items = calls;
		let items = vec![];

		// let items = Some(calls); // TODO:
		// Self { index, name, items }
		Self { index, name, items }
	}
}

#[cfg(test)]
impl Default for ReducedPallet {
	fn default() -> Self {
		Self { index: 42, name: "Foobar".into(), items: vec![] }
	}
}
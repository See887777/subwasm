use super::{calls::PalletId, changed_wapper::ChangedWrapper, reduced_pallet::*};
use super::reduced_runtime::ReducedRuntimeChange;

use comparable::MapChange;
use std::rc::Rc;


pub trait Compatible {
	/// This function reports whether the 2 runtimes APIs are compatible or not.
	/// This helps decide whether the runtime's `transaction_version` should be bumped or not.
	fn compatible(&self) -> bool;
}

/// This struct holds both the [ReducedRuntime] and its changes.
/// It allows computing stats about the amount of changes,
/// what has changed (or not) and making the decision about wether
/// the new runtime breaks API compatibility with the reference one.
pub struct DiffAnalyzer {
	pub changes: Rc<ChangedWrapper>,
}

impl DiffAnalyzer {
	pub fn new(changes: Rc<ChangedWrapper>) -> Self {
		Self { changes }
	}

	pub fn get_pallet_changes(
		&self,
		pallet_id: u32,
	) -> Option<&MapChange<PalletId, ReducedPalletDesc, Vec<ReducedPalletChange>>> {
		self.changes.get_pallet_changes_by_id(pallet_id)
	}
}

impl Compatible for DiffAnalyzer {
	fn compatible(&self) -> bool {
		match self.changes.0.changes {
			ReducedRuntimeChange::Extrinsic(extrinsic) => {
				extrinsic.iter().map(|p| match p {
					ReducedExtrinsicChange::Version(version) => {
						// match versiopn {

						// }
						// TODO
						true
					},
					ReducedExtrinsicChange::SignedExtensions(signed_extensions) => {
						// match signed_extensions {
							
							// }
						// TODO
						true
					},
				}).all(|x| x),
			},
			ReducedRuntimeChange::Pallets(pallets) => pallets
				.iter()
				.map(|p| match p {
					comparable::MapChange::Added(_key, _ddesc) => true,
					comparable::MapChange::Removed(_key) => false,
					comparable::MapChange::Changed(_key, change) => change.iter().map(|x| x.compatible()).all(|x| x),
				})
				.all(|x| x),
		}
	}
}

#[cfg(test)]
mod test_diffanalyzer {
	use super::*;
	use crate::differs::{reduced::reduced_diff_result::ReducedDiffResult, test_runtimes::*};
	use std::path::PathBuf;
	use wasm_loader::Source;
	use wasm_testbed::WasmTestBed;

	fn analyze(rf1: RuntimeFile, rf2: RuntimeFile) -> Option<DiffAnalyzer> {
		let a = get_runtime_file(rf1).expect("Runtime file should exist");
		let ra = WasmTestBed::new(&Source::File(a)).unwrap().metadata().into();
		let b = get_runtime_file(rf2).expect("Runtime file should exist");
		let rb = WasmTestBed::new(&Source::File(b)).unwrap().metadata().into();
		ReducedDiffResult::new(ra, rb).changes.map(DiffAnalyzer::new)
	}

	fn compare_runtimes_compatibility(runtime_a: PathBuf, runtime_b: PathBuf) -> bool {
		let a = WasmTestBed::new(&Source::File(runtime_a)).unwrap();
		let b = WasmTestBed::new(&Source::File(runtime_b)).unwrap();

		let ra = a.metadata().into();
		let rb = b.metadata().into();
		let res = ReducedDiffResult::new(ra, rb);

		match res.changes {
			Some(changes) => {
				let da = DiffAnalyzer::new(changes);
				println!("spec_version {:?} -> {:?}", a.core_version().spec_version, b.core_version().spec_version);
				println!(
					"transaction {:?} -> {:?}",
					a.core_version().transaction_version,
					b.core_version().transaction_version
				);
				let compatible = da.compatible();
				println!("compatible = {compatible:?}");
				compatible
			}
			None => {
				println!("No change found");
				true
			}
		}
	}

	#[test]
	#[ignore = "local data"]
	fn test_compatible_9260_9260() {
		assert!(compare_runtimes_compatibility(
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9260)).unwrap(),
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9260)).unwrap(),
		));
	}

	#[test]
	#[ignore = "local data"]
	fn test_compatible_9270_9270() {
		assert!(compare_runtimes_compatibility(
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9270)).unwrap(),
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9270)).unwrap(),
		));
	}

	#[test]
	#[ignore = "local data"]
	fn test_compatible_not_9260_9270() {
		assert!(!compare_runtimes_compatibility(
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9260)).unwrap(),
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9280)).unwrap(),
		));
	}

	#[test]
	#[ignore = "local data"]
	fn test_compatible_ksm_not_9280_9290() {
		assert!(!compare_runtimes_compatibility(
			get_runtime_file(RuntimeFile::new(Chain::Kusama, 14, 9280)).unwrap(),
			get_runtime_file(RuntimeFile::new(Chain::Kusama, 14, 9290)).unwrap(),
		));
	}

	#[test]
	#[ignore = "local data"]
	fn test_compatible_dot_not_9280_9290() {
		assert!(!compare_runtimes_compatibility(
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9280)).unwrap(),
			get_runtime_file(RuntimeFile::new(Chain::Polkadot, 14, 9290)).unwrap(),
		));
	}

	#[test]
	#[ignore = "local data"]
	fn test_changes_9280_9290() {
		let da =
			analyze(RuntimeFile::new(Chain::Polkadot, 14, 9280), RuntimeFile::new(Chain::Polkadot, 14, 9290)).unwrap();
		let pallet_system_changes = da.get_pallet_changes(0).unwrap();
		println!("pallet_system_changes = {pallet_system_changes:#?}");

		// There is a single change in the system pallet between 9280 and 9290: Constant: Version
		match pallet_system_changes {
			MapChange::Changed(k, changes) => {
				assert_eq!(&0, k);
				assert_eq!(1, changes.len());
				let change = &changes[0];
				assert!(change.compatible());
			}
			_ => panic!("Unexpected change while comparing 9280 and 9290"),
		}

		let pallet_balances_changes = da.get_pallet_changes(4).unwrap();
		println!("pallet_balances_changes = {pallet_balances_changes:#?}");

		// There is a single change in the balances pallet between 9280 and 9290: Calls: Signature changed
		match pallet_balances_changes {
			MapChange::Changed(k, changes) => {
				assert_eq!(&4, k);
				assert_eq!(1, changes.len());
				let change = &changes[0];
				assert!(!change.compatible());
			}
			_ => panic!("Unexpected change while comparing 9280 and 9290"),
		}
	}

	#[test]
	#[cfg(feature = "v13")]
	#[cfg(feature = "v14")]
	#[ignore = "local data"]
	fn test_different_variants_v13_v14() {
		let a = WasmTestBed::new(&Source::File(PathBuf::from(RUNTIME_V13_1))).unwrap();
		let b = WasmTestBed::new(&Source::File(PathBuf::from(RUNTIME_V14))).unwrap();
		let _differ = ReducedDiffer::new(a.metadata(), b.metadata());
	}

	#[test]
	#[cfg(feature = "v13")]
	#[ignore = "local data"]
	fn test_v13() {
		let a = WasmTestBed::new(&Source::File(PathBuf::from(RUNTIME_V13_1))).unwrap();
		let b = WasmTestBed::new(&Source::File(PathBuf::from(RUNTIME_V13_2))).unwrap();
		let comp = ReducedDiffer::compare(&a, &b);
	}

	#[test]
	#[cfg(feature = "v14")]
	#[ignore = "local data"]
	fn test_v14_polkadot_9290_9290() {
		let diff = analyze(RuntimeFile::new(Chain::Polkadot, 14, 9290), RuntimeFile::new(Chain::Polkadot, 14, 9290));
		assert!(diff.is_none());
	}

	#[test]
	#[cfg(all(feature = "v13", feature = "v14"))]
	#[ignore = "local data"]
	fn test_v14_polkadot_9100_9260() {
		let diff =
			analyze(RuntimeFile::new(Chain::Polkadot, 14, 9100), RuntimeFile::new(Chain::Polkadot, 14, 9260)).unwrap();

		assert!(!diff.compatible())
	}

	#[test]
	#[cfg(feature = "v14")]
	#[ignore = "local data"]
	fn test_v14_polkadot_9260_9270_full() {
		let diff =
			analyze(RuntimeFile::new(Chain::Polkadot, 14, 9260), RuntimeFile::new(Chain::Polkadot, 14, 9270)).unwrap();

		assert!(!diff.compatible())
	}

	#[test]
	#[ignore = "local data"]
	#[should_panic]
	fn test_unsupported_variants() {
		let diff =
			analyze(RuntimeFile::new(Chain::Polkadot, 12, 9000), RuntimeFile::new(Chain::Polkadot, 12, 9000)).unwrap();

		assert!(!diff.compatible())
	}

	#[test]
	#[cfg(feature = "v14")]
	#[ignore = "local data"]
	fn test_v14_polkadot_9280_9290_full() {
		let diff =
			analyze(RuntimeFile::new(Chain::Polkadot, 14, 9280), RuntimeFile::new(Chain::Polkadot, 14, 9290)).unwrap();
		assert!(!diff.compatible());

		println!("changes = {:?}", diff.changes);
		// assert_eq!(12, diff.changes.get )

		// todo: add folling expectations to the test. Those are tested in test_changes_9280_9290
		// all the changes below are SIG changes
		// - TEST: Summary pallet  | compat change  | breaking change
		// 			[System] | 0              | sig: 1 [blockWeight] type: FrameSupportWeightsPerDispatchClassU64 -> FrameSupportWeightsPerDispatchClassWeight
		// 		[Scheduler]  | 0 			  | 0
		// 		 [Preimage] | 0 			  | 0
		// 			 [Babe] | 0 			  | 0
		// 		[Timestamp] | 0 			  | 0
		// 		  [Indices] | 0 			  | sig: 2 (transfer + forceTransfer)
		// 		 [Balances] | 0 			  | 0
		// 	   [Authorship] | 0 			  | 0
		// 		  [Staking] | 0 			  | 0
		// 		  [Session] | 0 			  | 0
		// 		  [Grandpa] | 0 			  | 0
		// 		 [ImOnline] | 0 			  | 0
		// 		[Democracy] | 0    			  | 3
		// 		  [Council] | 0    			  | 1
		// [TechnicalCommittee] | 0    		  | 1
		//  [PhragmenElection] | 0    		  | 0
		// [TechnicalMembership] | 0    	  | 5
		// 		 [Treasury] | 0    			  | 0
		// 		   [Claims] | 0    			  | 0
		// 		  [Vesting] | 0    			  | 0
		// 		 [Identity] | 0    			  | 2
		// 			[Proxy] | 0    			  | 8
		// 		 [Multisig] | 0    			  | 2
		// 		 [Bounties] | 0    			  | 0
		// 	[ChildBounties] | 0    			  | 0
		// 			 [Tips] | 0    			  | 2
		// [ElectionProviderMultiPhase] | 0   | 0
		// 		[VoterList] | 0    			  | 2
		//   [NominationPools] | 0    		  | 3
		// 	[Configuration] idx: 51 | 0    	  | 0
		// 	   [setUmpServiceTotalWeight] | 0  | 1
		// 	  [setUmpMaxIndividualWeight]| 0    | 1
		// 	 [ParaInherent] idx: 54 | 0    	  | 0
		// 			[Paras] idx: 56 | 0    	  | 0
		// 	  [Initializer] idx: 57 | 0    	  | 0
		// 			  [Ump] idx: 59| 0    	  | 0
		// 			  [serviceOverweight] | 0  | 1
		// 			 [Hrmp] idx: 60 | 0    	  | 0
		// 	[ParasDisputes] idx: 62| 0    	  | 0
		// 		[Registrar] idx: 70 | 0   	  | 0
		// 			[Slots] idx: 71 | 0   	  | 0
		// 		 [Auctions] idx: 72| 0    	  | 0
		// 		[Crowdloan] idx: 73 | 0    	  | 0
		// 		[XcmPallet] idx: 99 | 0    	  | 1
	}
}
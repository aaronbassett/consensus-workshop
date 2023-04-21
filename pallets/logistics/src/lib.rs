#![cfg_attr(not(feature = "std"), no_std)]

// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub struct Coords {
		lat: u32,
		lng: u32,
	}

	#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Shipment<T: Config> {
		id: u64,
		shipped_by: T::AccountId,
		received_by: T::AccountId,
		received_at: Coords,
		destination: u64,
		delivered: bool,
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	pub type Shipments<T> = CountedStorageMap<_, Blake2_128Concat, u64, Shipment<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Shipment received [shipment_id, shipped_by, received_by]
		ShipmentReceived { shipment_id: u64, received_by: T::AccountId, received_at: Coords },

		/// Shipment has been delivered [shipment_id]
		ShipmentDelivered { shipment_id: u64 },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// No shipment found for supplied id
		ShipmentDoesNotExist,
		/// Cannot create shipment with duplicate id
		DuplicateShipment,
		/// Cannot modify shipment that has been delivered
		ShipmentNotInTransit,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}

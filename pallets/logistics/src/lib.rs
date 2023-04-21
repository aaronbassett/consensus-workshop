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
		received_on: T::BlockNumber,
		destination: u64,
		delivered: bool,
	}

	impl<T: Config> Shipment<T> {
		pub fn new(
			shipment_id: u64,
			shipped_by: T::AccountId,
			received_by: T::AccountId,
			received_at: Coords,
			destination: u64,
		) -> Self {
			Shipment {
				id: shipment_id,
				shipped_by,
				received_by,
				received_at,
				received_on: frame_system::Pallet::<T>::block_number(),
				destination,
				delivered: false,
			}
		}
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	pub type Shipments<T> = CountedStorageMap<_, Blake2_128Concat, u64, Shipment<T>>;

	#[pallet::storage]
	pub type DeliveredLog<T> = StorageValue<_, BoundedVec<u64, ConstU32<100>>, ValueQuery>;

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
		/// Delivered log is full
		DeliveredLogOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			for shipment_id in DeliveredLog::<T>::get().iter() {
				Shipments::<T>::remove(shipment_id);
			}
			DeliveredLog::<T>::kill();
			Weight::zero()
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn begin_transit(
			origin: OriginFor<T>,
			shipment_id: u64,
			shipped_by: T::AccountId,
			received_at: Coords,
			destination: u64,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let received_by = ensure_signed(origin)?;

			ensure!(!Shipments::<T>::contains_key(&shipment_id), Error::<T>::DuplicateShipment);

			Shipments::<T>::insert(
				&shipment_id,
				Shipment::new(
					shipment_id,
					shipped_by.clone(),
					received_by.clone(),
					received_at.clone(),
					destination,
				),
			);

			Self::deposit_event(Event::ShipmentReceived { shipment_id, received_by, received_at });

			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn shipment_received(
			origin: OriginFor<T>,
			shipment_id: u64,
			received_at: Coords,
		) -> DispatchResult {
			let received_by = ensure_signed(origin)?;

			Shipments::<T>::try_mutate(&shipment_id, |shipment| -> DispatchResult {
				match shipment {
					Some(s) if !s.delivered => {
						s.received_by = received_by.clone();
						s.received_at = received_at.clone();
						s.received_on = frame_system::Pallet::<T>::block_number();
					},
					Some(s) if s.delivered => return Err(Error::<T>::ShipmentNotInTransit.into()),
					_ => return Err(Error::<T>::ShipmentDoesNotExist.into()),
				}
				Ok(())
			})?;

			Self::deposit_event(Event::ShipmentReceived { shipment_id, received_by, received_at });

			Ok(())
		}

		#[pallet::call_index(20)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn shipment_delivered(
			origin: OriginFor<T>,
			shipment_id: u64,
			received_at: Coords,
		) -> DispatchResult {
			let received_by = ensure_signed(origin)?;

			Shipments::<T>::try_mutate(&shipment_id, |shipment| -> DispatchResult {
				match shipment {
					Some(s) if !s.delivered => {
						s.received_by = received_by;
						s.received_at = received_at;
						s.received_on = frame_system::Pallet::<T>::block_number();
						s.delivered = true;
					},
					Some(s) if s.delivered => return Err(Error::<T>::ShipmentNotInTransit.into()),
					_ => return Err(Error::<T>::ShipmentDoesNotExist.into()),
				}

				DeliveredLog::<T>::try_append(shipment_id)
					.map_err(|_| Error::<T>::DeliveredLogOverflow)?;

				Ok(())
			})?;

			Self::deposit_event(Event::ShipmentDelivered { shipment_id });

			Ok(())
		}
	}
}

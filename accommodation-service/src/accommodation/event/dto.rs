use std::any::Any;

use kafka_schema_accommodation::schema_create_accommodation::CreateAccommodationAvro;
use kafka_schema_accommodation::schema_create_room_type::CreateRoomTypeAvro;
use kafka_schema_accommodation::schema_delete_room_type::DeleteRoomTypeAvro;
use kafka_schema_accommodation::schema_update_accommodation::UpdateAccommodationAvro;
use kafka_schema_accommodation::schema_update_room_type::UpdateRoomTypeAvro;
use kafka_schema_accommodation::AccommodationAddressAvro;
use kafka_schema_accommodation::BedTypeAvro;
use kafka_schema_accommodation::IsoCountryCodeEnumAvro;

use crate::accommodation::model::Accommodation;
use crate::accommodation::model::BedType;
use crate::accommodation::model::RoomType;
use crate::common::model::IsoCountryCodeEnum;
use crate::event::service::dto::SerializableEventDto;

impl SerializableEventDto for Accommodation {
    fn event_type(&self, event_type: String) -> String {
        event_type
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<Accommodation> for CreateAccommodationAvro {
    fn from(accommodation: Accommodation) -> Self {
        CreateAccommodationAvro {
            identifier: format!("{}", accommodation.id),
            name: accommodation.name,
            description: accommodation.description,
            address: AccommodationAddressAvro {
                street: accommodation.address.street,
                house_number: accommodation.address.house_number,
                zip_code: accommodation.address.zip_code,
                city: accommodation.address.city,
                area: accommodation.address.area,
                country: match accommodation.address.country {
                    IsoCountryCodeEnum::DE => IsoCountryCodeEnumAvro::DE,
                    IsoCountryCodeEnum::US => IsoCountryCodeEnumAvro::US,
                },
            },
        }
    }
}

impl From<Accommodation> for UpdateAccommodationAvro {
    fn from(accommodation: Accommodation) -> Self {
        UpdateAccommodationAvro {
            identifier: format!("{}", accommodation.id),
            name: accommodation.name,
            description: accommodation.description,
            address: AccommodationAddressAvro {
                street: accommodation.address.street,
                house_number: accommodation.address.house_number,
                zip_code: accommodation.address.zip_code,
                city: accommodation.address.city,
                area: accommodation.address.area,
                country: match accommodation.address.country {
                    IsoCountryCodeEnum::DE => IsoCountryCodeEnumAvro::DE,
                    IsoCountryCodeEnum::US => IsoCountryCodeEnumAvro::US,
                },
            },
        }
    }
}

impl SerializableEventDto for RoomType {
    fn event_type(&self, event_type: String) -> String {
        event_type
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<RoomType> for CreateRoomTypeAvro {
    fn from(r: RoomType) -> Self {
        CreateRoomTypeAvro {
            accommodation_id: r.accommodation_id.to_string(),
            identifier: r.id.to_string(),
            size: r.size,
            balcony: r.balcony,
            bed_type: match r.bed_type {
                BedType::Single => BedTypeAvro::Single,
                BedType::TwinSingle => BedTypeAvro::TwinSingle,
                BedType::Double => BedTypeAvro::Double,
                BedType::King => BedTypeAvro::King,
            },
            tv: r.tv,
            wifi: r.wifi,
        }
    }
}

impl From<RoomType> for UpdateRoomTypeAvro {
    fn from(r: RoomType) -> Self {
        UpdateRoomTypeAvro {
            accommodation_id: r.accommodation_id.to_string(),
            identifier: r.id.to_string(),
            size: r.size,
            balcony: r.balcony,
            bed_type: match r.bed_type {
                BedType::Single => BedTypeAvro::Single,
                BedType::TwinSingle => BedTypeAvro::TwinSingle,
                BedType::Double => BedTypeAvro::Double,
                BedType::King => BedTypeAvro::King,
            },
            tv: r.tv,
            wifi: r.wifi,
        }
    }
}

impl From<RoomType> for DeleteRoomTypeAvro {
    fn from(r: RoomType) -> Self {
        DeleteRoomTypeAvro {
            accommodation_id: r.accommodation_id.to_string(),
            identifier: r.id.to_string(),
        }
    }
}

#![allow(dead_code)]

pub mod oscal {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.common.v1.rs"));
        }
    }

    pub mod catalog {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.catalog.v1.rs"));
        }
    }

    pub mod profile {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.profile.v1.rs"));
        }
    }

    pub mod component_definition {
        pub mod v1 {
            include!(concat!(
                env!("OUT_DIR"),
                "/oscal.component_definition.v1.rs"
            ));
        }
    }

    pub mod ssp {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.ssp.v1.rs"));
        }
    }

    pub mod assessment_plan {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.assessment_plan.v1.rs"));
        }
    }

    pub mod assessment_results {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.assessment_results.v1.rs"));
        }
    }

    pub mod poam {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.poam.v1.rs"));
        }
    }

    pub mod mapping {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.mapping.v1.rs"));
        }
    }

    pub mod services {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/oscal.services.v1.rs"));
        }
    }
}

#[cfg(test)]
mod tests {
    use prost_reflect::DescriptorPool;

    use super::oscal::mapping::v1::{ControlMapping, Map, MappingCollection};

    #[test]
    fn descriptor_carries_oscal_1_2_schema_additions() {
        let pool = DescriptorPool::decode(crate::PROTO_DESCRIPTOR_SET)
            .expect("embedded descriptor set must decode");

        for (message, fields) in [
            (
                "oscal.assessment_plan.v1.ControlSelection",
                &["include_all"][..],
            ),
            (
                "oscal.assessment_results.v1.ControlSelection",
                &["include_all"][..],
            ),
            (
                "oscal.mapping.v1.MappingCollection",
                &["provenance", "mappings", "back_matter"][..],
            ),
        ] {
            let descriptor = pool
                .get_message_by_name(message)
                .unwrap_or_else(|| panic!("missing descriptor for {message}"));
            for field in fields {
                assert!(
                    descriptor.get_field_by_name(field).is_some(),
                    "{message} is missing {field}"
                );
            }
        }
        assert!(pool
            .get_message_by_name("oscal.mapping.v1.ControlMapping")
            .is_some());
        for message in [
            "oscal.services.v1.FetchExternalEvidenceRequest",
            "oscal.services.v1.FetchExternalEvidenceResponse",
            "oscal.services.v1.EvidenceFetchAudit",
            "oscal.services.v1.ListFetchEventsRequest",
            "oscal.services.v1.ListFetchEventsResponse",
        ] {
            assert!(
                pool.get_message_by_name(message).is_some(),
                "missing transparency evidence message {message}"
            );
        }
        let transparency = pool
            .get_service_by_name("oscal.services.v1.TransparencyExchangeService")
            .expect("missing transparency exchange service descriptor");
        for method in ["FetchExternalEvidence", "ListFetchEvents"] {
            assert!(
                transparency
                    .methods()
                    .any(|candidate| candidate.name() == method),
                "missing transparency exchange method {method}"
            );
        }
        assert_eq!(crate::OSCAL_SCHEMA_VERSION, "1.2.3");
        assert!(crate::OSCAL_SCHEMA_MANIFEST_SHA256.starts_with("sha256:"));
    }

    #[test]
    #[allow(deprecated)]
    fn mapping_count_prefers_released_shape_with_legacy_fallback() {
        let mut mapping = MappingCollection {
            maps: vec![Map::default(), Map::default()],
            ..Default::default()
        };
        assert_eq!(crate::cli::mapping_count(&mapping), 2);

        mapping.mappings = vec![ControlMapping::default()];
        assert_eq!(crate::cli::mapping_count(&mapping), 1);
    }
}

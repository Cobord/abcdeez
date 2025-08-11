pub mod preregistration;
pub mod irb;
pub mod audit_trail;
pub mod citation_manager;

pub use preregistration::*;
pub use irb::{ConsentTemplate, IRBApplication, IRBComplianceGenerator, StudySummary};
pub use audit_trail::{
    Actor, ActorType, AuditConfiguration, AuditLevel, AuditTrailManager, EventType, Operation,
    Outcome, Resource,
};
pub use citation_manager::{
    Author, BibliographyFormat, BibliographyStyle, CitationManager, MethodologyReport, Publication,
    Reference, ReferenceType,
};
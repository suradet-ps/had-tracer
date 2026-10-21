use serde::{Deserialize, Serialize};

use crate::model::{new_id, now_rfc3339};

/// Type of clinical tracer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseType {
    Individual,
    System,
    ProgramSpecific,
}

impl CaseType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Individual => "individual",
            Self::System => "system",
            Self::ProgramSpecific => "program_specific",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "system" => Self::System,
            "program_specific" => Self::ProgramSpecific,
            _ => Self::Individual,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::Individual => "ตามรอยผู้ป่วยรายบุคคล",
            Self::System => "ตามรอยระบบงาน",
            Self::ProgramSpecific => "ตามรอยเฉพาะโครงการ",
        }
    }
}

/// Tracer case lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracerStatus {
    Open,
    InReview,
    Closed,
}

impl TracerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Closed => "closed",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "in_review" => Self::InReview,
            "closed" => Self::Closed,
            _ => Self::Open,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::Open => "เปิด",
            Self::InReview => "กำลังทบทวน",
            Self::Closed => "ปิด",
        }
    }
}

/// Seven-phase tracer worksheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseCode {
    PreEncounter,
    Prescribing,
    Verification,
    Preparation,
    Administration,
    Monitoring,
    Handoff,
}

impl PhaseCode {
    pub const ALL: [PhaseCode; 7] = [
        Self::PreEncounter,
        Self::Prescribing,
        Self::Verification,
        Self::Preparation,
        Self::Administration,
        Self::Monitoring,
        Self::Handoff,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreEncounter => "pre_encounter",
            Self::Prescribing => "prescribing",
            Self::Verification => "verification",
            Self::Preparation => "preparation",
            Self::Administration => "administration",
            Self::Monitoring => "monitoring",
            Self::Handoff => "handoff",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "prescribing" => Self::Prescribing,
            "verification" => Self::Verification,
            "preparation" => Self::Preparation,
            "administration" => Self::Administration,
            "monitoring" => Self::Monitoring,
            "handoff" => Self::Handoff,
            _ => Self::PreEncounter,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::PreEncounter => "ก่อนรับการรักษา",
            Self::Prescribing => "การสั่งใช้ยา",
            Self::Verification => "การตรวจสอบคำสั่ง",
            Self::Preparation => "การเตรียมและจ่ายยา",
            Self::Administration => "การบริหารยา",
            Self::Monitoring => "การติดตามผล",
            Self::Handoff => "การส่งต่อและจำหน่าย",
        }
    }

    pub fn focus_th(self) -> &'static str {
        match self {
            Self::PreEncounter => "ประวัติ การใช้ยาเดิม การแพ้ยา และค่าพื้นฐาน",
            Self::Prescribing => "ข้อบ่งใช้ ขนาด วิถีทาง ระยะเวลา และการปรับตามสภาพผู้ป่วย",
            Self::Verification => "การทบทวนของเภสัชกร การสื่อสาร และการตรวจสอบความซ้ำซ้อน",
            Self::Preparation => "การเจือจาง ฉลาก การแยกเก็บ การเข้าถึง และ independent double check",
            Self::Administration => "6R, เครื่อง infusion pump, สายให้ยา และการเฝ้าระวังระหว่างให้ยา",
            Self::Monitoring => "ผลแล็บ สัญญาณชีพ อาการไม่พึงประสงค์ และการ escalate",
            Self::Handoff => "การส่งต่อเวร ยาจำหน่าย การให้ความรู้ และการนัดติดตาม",
        }
    }

    pub fn questions(self) -> &'static [&'static str] {
        match self {
            Self::PreEncounter => &[
                "มีประวัติแพ้ยาหรือ ADR ที่เกี่ยวข้องกับ HAD หรือไม่",
                "มีการสอบถามยาเดิมและตรวจสอบขนาดยาที่บ้านหรือไม่",
                "มีค่าพื้นฐานที่จำเป็นก่อนเริ่มยา เช่น INR, aPTT, K, creatinine หรือ glucose หรือไม่",
                "ผู้ป่วยมีความเสี่ยงสูง เช่น renal impairment, อายุมาก, น้ำหนักน้อย หรือตั้งครรภ์ หรือไม่",
            ],
            Self::Prescribing => &[
                "มีข้อบ่งใช้ชัดเจนและบันทึกในเวชระเบียนหรือไม่",
                "ขนาด วิถีทาง ความถี่ และระยะเวลาสอดคล้องกับแนวทางหรือไม่",
                "มีการปรับขนาดตาม renal หรือ hepatic function หรือไม่",
                "มีการตรวจสอบยาที่มี interaction หรือ duplication หรือไม่",
                "มี target range หรือ hold criteria ระบุชัดเจนหรือไม่",
            ],
            Self::Verification => &[
                "เภสัชกรตรวจสอบคำสั่งก่อน dose แรกหรือไม่",
                "มีการสื่อสารข้อสงสัยและได้รับการตอบกลับที่ชัดเจนหรือไม่",
                "มีการตรวจสอบ LASA และความแรงที่อาจสับสนหรือไม่",
                "มีการทวนสอบผู้ป่วยกลุ่มเสี่ยงสูงก่อนจ่ายยาหรือไม่",
            ],
            Self::Preparation => &[
                "เตรียมยาในพื้นที่และสภาพแวดล้อมที่เหมาะสมหรือไม่",
                "การเจือจาง ความเข้มข้น และ diluent ถูกต้องหรือไม่",
                "ฉลากมีชื่อยา ความแรง ปริมาณ วิธีใช้ และคำเตือนครบหรือไม่",
                "มีการแยกเก็บและจำกัดการเข้าถึงหรือไม่",
                "มี independent double check โดยบุคคลที่สองหรือไม่",
            ],
            Self::Administration => &[
                "ตรวจสอบผู้ป่วย ยา ขนาด วิถีทาง เวลา และเอกสารครบถ้วนหรือไม่",
                "ใช้ infusion pump หรือ syringe pump และตั้งค่าถูกต้องหรือไม่",
                "มีการตรวจสอบสายให้ยา ตำแหน่ง และ patency หรือไม่",
                "มีการเฝ้าระวังระหว่างให้ยาและบันทึกผลหรือไม่",
                "มีผู้ลงนามเป็นพยานในการตรวจสอบซ้ำหรือไม่",
            ],
            Self::Monitoring => &[
                "มีการตรวจ lab ตามความถี่ที่กำหนดหรือไม่",
                "มีการติดตามสัญญาณชีพและอาการไม่พึงประสงค์หรือไม่",
                "มีการปรับขนาดหรือหยุดยาตามเกณฑ์หรือไม่",
                "มียาแก้พิษหรือ reversal agent พร้อมใช้หรือไม่",
                "มีการ escalate เมื่อพบผลผิดปกติหรือไม่",
            ],
            Self::Handoff => &[
                "มีการส่งต่อข้อมูล HAD ที่สำคัญในเวรหรือไม่",
                "ยาที่ต้องใช้ต่อที่บ้านได้รับการทบทวนและอธิบายหรือไม่",
                "ผู้ป่วยหรือผู้ดูแลเข้าใจ warning signs หรือไม่",
                "มีการนัดติดตามผล lab หรืออาการหรือไม่",
                "มีข้อมูลส่งต่อไปยัง primary care หรือไม่",
            ],
        }
    }
}

/// A tracer review case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerCase {
    pub id: String,
    pub case_type: CaseType,
    pub title: String,
    pub purpose: String,
    pub hn: String,
    pub an: Option<String>,
    pub vn: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_ref: Option<String>,
    pub lead_user: Option<String>,
    pub status: TracerStatus,
    pub opened_at: String,
    pub closed_at: Option<String>,
}

impl TracerCase {
    pub fn new(
        case_type: CaseType,
        title: String,
        purpose: String,
        hn: String,
        lead_user: Option<String>,
    ) -> Self {
        Self {
            id: new_id(),
            case_type,
            title,
            purpose,
            hn,
            an: None,
            vn: None,
            trigger_type: None,
            trigger_ref: None,
            lead_user,
            status: TracerStatus::Open,
            opened_at: now_rfc3339(),
            closed_at: None,
        }
    }
}

/// One phase row inside a case.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerPhase {
    pub id: String,
    pub case_id: String,
    pub phase_code: PhaseCode,
    pub status: String,
    pub summary: String,
}

/// A checklist answer inside a phase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerCheck {
    pub id: String,
    pub case_id: String,
    pub phase_id: String,
    pub standard_code: Option<String>,
    pub question: String,
    pub expected: String,
    pub observed: String,
    pub gap: String,
    pub evidence_summary: String,
    pub finding_id: Option<String>,
}

/// Severity of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl FindingSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "high" => Self::High,
            "critical" => Self::Critical,
            "low" => Self::Low,
            _ => Self::Medium,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::Low => "ต่ำ",
            Self::Medium => "ปานกลาง",
            Self::High => "สูง",
            Self::Critical => "วิกฤต",
        }
    }
}

/// Finding lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Open,
    Assigned,
    InProgress,
    Implemented,
    Verified,
    Closed,
    Reopened,
}

impl FindingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Assigned => "assigned",
            Self::InProgress => "in_progress",
            Self::Implemented => "implemented",
            Self::Verified => "verified",
            Self::Closed => "closed",
            Self::Reopened => "reopened",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "assigned" => Self::Assigned,
            "in_progress" => Self::InProgress,
            "implemented" => Self::Implemented,
            "verified" => Self::Verified,
            "closed" => Self::Closed,
            "reopened" => Self::Reopened,
            _ => Self::Open,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::Open => "เปิด",
            Self::Assigned => "มอบหมาย",
            Self::InProgress => "กำลังดำเนินการ",
            Self::Implemented => "ดำเนินการแล้ว",
            Self::Verified => "ตรวจสอบผลแล้ว",
            Self::Closed => "ปิด",
            Self::Reopened => "เปิดใหม่",
        }
    }
}

/// Human-factors cause category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseCategory {
    Patient,
    Task,
    Individual,
    Team,
    Environment,
    Equipment,
    Organization,
    Policy,
    InformationTechnology,
}

impl CauseCategory {
    pub const ALL: [CauseCategory; 9] = [
        Self::Patient,
        Self::Task,
        Self::Individual,
        Self::Team,
        Self::Environment,
        Self::Equipment,
        Self::Organization,
        Self::Policy,
        Self::InformationTechnology,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Patient => "patient",
            Self::Task => "task",
            Self::Individual => "individual",
            Self::Team => "team",
            Self::Environment => "environment",
            Self::Equipment => "equipment",
            Self::Organization => "organization",
            Self::Policy => "policy",
            Self::InformationTechnology => "information_technology",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "patient" => Self::Patient,
            "task" => Self::Task,
            "individual" => Self::Individual,
            "team" => Self::Team,
            "environment" => Self::Environment,
            "equipment" => Self::Equipment,
            "organization" => Self::Organization,
            "policy" => Self::Policy,
            "information_technology" => Self::InformationTechnology,
            _ => Self::Task,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::Patient => "ปัจจัยผู้ป่วย",
            Self::Task => "ลักษณะงาน",
            Self::Individual => "ปัจจัยบุคคล",
            Self::Team => "ทีมและสื่อสาร",
            Self::Environment => "สภาพแวดล้อม",
            Self::Equipment => "เครื่องมือ",
            Self::Organization => "องค์กร",
            Self::Policy => "นโยบาย",
            Self::InformationTechnology => "เทคโนโลยีสารสนเทศ",
        }
    }
}

/// A gap found during tracer review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: String,
    pub case_id: String,
    pub phase_id: Option<String>,
    pub title: String,
    pub description: String,
    pub standard_code: Option<String>,
    pub severity: FindingSeverity,
    pub likelihood: i64,
    pub risk_score: i64,
    pub cause_category: CauseCategory,
    pub immediate_action: String,
    pub recommended_action: String,
    pub status: FindingStatus,
    pub created_at: String,
}

impl Finding {
    /// Build a finding with a 1 to 5 likelihood and severity-based impact.
    pub fn new(
        case_id: String,
        phase_id: Option<String>,
        title: String,
        description: String,
        severity: FindingSeverity,
        likelihood: i64,
        cause_category: CauseCategory,
    ) -> Self {
        let impact = match severity {
            FindingSeverity::Low => 1,
            FindingSeverity::Medium => 2,
            FindingSeverity::High => 4,
            FindingSeverity::Critical => 5,
        };
        Self {
            id: new_id(),
            case_id,
            phase_id,
            title,
            description,
            standard_code: None,
            severity,
            likelihood: likelihood.clamp(1, 5),
            risk_score: (impact * likelihood.clamp(1, 5)).clamp(1, 25),
            cause_category,
            immediate_action: String::new(),
            recommended_action: String::new(),
            status: FindingStatus::Open,
            created_at: now_rfc3339(),
        }
    }
}

/// CAPA lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapaStatus {
    Open,
    Assigned,
    InProgress,
    Implemented,
    Verified,
    Closed,
    Reopened,
}

impl CapaStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Assigned => "assigned",
            Self::InProgress => "in_progress",
            Self::Implemented => "implemented",
            Self::Verified => "verified",
            Self::Closed => "closed",
            Self::Reopened => "reopened",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(value: &str) -> Self {
        match value {
            "assigned" => Self::Assigned,
            "in_progress" => Self::InProgress,
            "implemented" => Self::Implemented,
            "verified" => Self::Verified,
            "closed" => Self::Closed,
            "reopened" => Self::Reopened,
            _ => Self::Open,
        }
    }

    pub fn label_th(self) -> &'static str {
        match self {
            Self::Open => "เปิด",
            Self::Assigned => "มอบหมาย",
            Self::InProgress => "กำลังดำเนินการ",
            Self::Implemented => "ดำเนินการแล้ว",
            Self::Verified => "ตรวจสอบผลแล้ว",
            Self::Closed => "ปิด",
            Self::Reopened => "เปิดใหม่",
        }
    }
}

/// Corrective and preventive action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capa {
    pub id: String,
    pub finding_id: String,
    pub action: String,
    pub owner: String,
    pub due_date: Option<String>,
    pub status: CapaStatus,
    pub verification_method: String,
    pub verification_date: Option<String>,
    pub effectiveness_result: String,
    pub evidence: String,
    pub created_at: String,
    pub closed_at: Option<String>,
}

/// Aggregate returned by the tracer detail screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerWorkspace {
    pub case: TracerCase,
    pub phases: Vec<TracerPhase>,
    pub checks: Vec<TracerCheck>,
    pub findings: Vec<Finding>,
    pub capas: Vec<Capa>,
}

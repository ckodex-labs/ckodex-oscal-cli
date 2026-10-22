# Frontier AI & MITRE ATLAS Threat Matrix Guide

This guide details the high-assurance NIST OSCAL v1.2.3 components, threat catalogs, profiles, and crosswalks provided by Mizan for governing Frontier AI systems (Anthropic, Google, OpenAI, Cohere) against the MITRE ATLAS adversarial threat framework.

---

## 1. Overview of Artifacts

All artifacts reside in [`examples/usecases/`](../examples/usecases/) and adhere strictly to official NIST OSCAL v1.2.3 schemas:

| Artifact | OSCAL Model | Root Key | Description |
| :--- | :--- | :--- | :--- |
| `anthropic-claude-component.json` | Component Definition | `component-definition` | Anthropic Claude 3.5 / 3.7 and ClaudeGov on AWS GovCloud with ASL-2/ASL-3, Constitutional AI, and FedRAMP High controls. |
| `google-gemini-vertex-component.json` | Component Definition | `component-definition` | Google Cloud Vertex AI and Gemini 1.5 Pro with SAIF (Secure AI Framework), VPC Service Controls, and CMEK. |
| `openai-gpt4-azure-component.json` | Component Definition | `component-definition` | OpenAI GPT-4o deployed via Azure OpenAI Service with FedRAMP High JAB P-ATO, Azure Private Link, and Content Safety. |
| `cohere-north-component.json` | Component Definition | `component-definition` | Cohere North 2 Sovereign multi-cloud deployment with zero data retention, data residency, and isolated hardware enclaves. |
| `mitre-atlas-catalog.json` | Control Catalog | `catalog` | Official MITRE ATLAS (Adversarial Threat Landscape for AI Systems) catalog covering tactics AML.TA0001 through AML.TA0011. |
| `nist-to-mitre-atlas-crosswalk.json` | Mapping Collection | `mapping-collection` | Bidirectional crosswalk mapping NIST SP 800-53 Rev 5 controls to MITRE ATLAS threat techniques and NIST AI RMF. |
| `frontier-ai-mitre-profile.json` | Profile Baseline | `profile` | Tailored baseline importing NIST SP 800-53 Rev 5 Moderate and setting AI-specific parameter constraints. |

---

## 2. Frontier AI Component Definitions

### A. Anthropic Claude Enterprise & ClaudeGov
- **Target File**: `examples/usecases/anthropic-claude-component.json`
- **Key Control Implementations**:
  - `ac-2`: Role-based access control for API token lifecycle and workspace boundaries.
  - `ac-3`: Constitutional AI policy guardrails governing real-time model completions.
  - `ac-4`: Strict isolation between tenant prompts, context windows, and training pipelines.
  - `au-2`: Granular audit logging of all inference calls and safety classifier triggers.
  - `ia-2`: Mutual TLS (mTLS) and scoped API key authentication.
  - `sc-7`: Egress traffic filtering preventing unauthorized external tool calls.
  - `sc-13`: FIPS 140-3 validated cryptographic protection for model weights and transit data.
  - `si-4`: Real-time monitoring for adversarial prompt injection and extraction attacks.
  - `si-10`: Strict input sanitization and token boundary validation.
  - `sa-11`: Frontier red-teaming attestations and Responsible Scaling Policy (RSP) ASL compliance.

### B. Google Cloud Vertex AI & Gemini 1.5 Pro
- **Target File**: `examples/usecases/google-gemini-vertex-component.json`
- **Key Control Implementations**:
  - `ac-4`: VPC Service Controls establishing a strict cryptographic perimeter around model APIs.
  - `ac-6`: Least privilege execution for custom model fine-tuning jobs.
  - `au-3`: Cloud Audit Logs integrated with immutable BigQuery storage.
  - `sc-8`: Application Layer Transport Security (ALTS) and TLS 1.3 across model clusters.
  - `sc-28`: Customer-Managed Encryption Keys (CMEK) via Cloud KMS for data at rest.
  - `si-3`: Malicious payload detection and grounding safety filters against untrusted sources.
  - `si-7`: SynthID digital watermarking verifying synthetic text and image provenance.
  - `mp-6`: Ephemeral memory execution with verified zero customer data retention.

### C. OpenAI GPT-4o & Azure OpenAI Service
- **Target File**: `examples/usecases/openai-gpt4-azure-component.json`
- **Key Control Implementations**:
  - `ac-17`: Remote access restricted exclusively via Azure Private Link endpoints.
  - `ac-18`: Wireless and public ingress/egress lockouts.
  - `au-6`: Automated anomaly detection and security telemetry review.
  - `cm-8`: Automated hardware and model container inventory tracking.
  - `ia-4`: Microsoft Entra ID Managed Identity token handling.
  - `sc-12`: Cryptographic key establishment using Azure Dedicated HSM / Key Vault.
  - `si-4`: Azure AI Content Safety Prompt Shields detecting jailbreaks and injection attempts.
  - `ra-3`: Preparedness Framework evaluations covering CBRN, cyberattack, and autonomy risks.

### D. Cohere North 2 Sovereign AI
- **Target File**: `examples/usecases/cohere-north-component.json`
- **Key Control Implementations**:
  - `sc-28`: Cryptographic data isolation ensuring zero customer data retention (ZDR).
  - `ac-4`: National boundary enforcement and strictly bounded sovereign data residency.
  - `sc-13`: Hardware-enforced Confidential Computing (AMD SEV-SNP / NVIDIA Confidential GPU).
  - `si-4`: Real-time semantic audit logging of inference tensors and embeddings.

---

## 3. MITRE ATLAS AI Threat Matrix Catalog

The MITRE ATLAS catalog (`examples/usecases/mitre-atlas-catalog.json`) structures AI adversarial techniques into official OSCAL control groups matching MITRE tactics:

1. **Reconnaissance (`AML.TA0002`)**: Active scanning of model endpoints, architecture probing.
2. **Resource Development (`AML.TA0003`)**: Acquisition of adversarial training sets, prompt templates.
3. **Initial Access (`AML.TA0004`)**:
   - `AML.T0051`: Direct & Indirect LLM Prompt Injection.
   - `AML.T0054`: LLM Jailbreak & Behavioral Override.
   - `AML.T0010`: ML Supply Chain Poisoning.
4. **ML Attack Staging (`AML.TA0001`)**:
   - `AML.T0018`: Adversarial Data Poisoning.
   - `AML.T0048`: Inversion & Training Reconstruction.
5. **Exfiltration (`AML.TA0009`)**:
   - `AML.T0044`: Model Extraction & Weight Theft.
   - `AML.T0025`: Training Data Scraping & Inference Extraction.
6. **Impact (`AML.TA0011`)**:
   - `AML.T0043`: Insecure Output Handling & Autonomous Agent Hijacking.
   - `AML.T0029`: Denial of ML Service / Computational Exhaustion.

---

## 4. NIST SP 800-53 to MITRE ATLAS Crosswalk

The bidirectional crosswalk (`examples/usecases/nist-to-mitre-atlas-crosswalk.json`) uses the official OSCAL v1.2.3 `mapping-collection` model to link threat techniques to compensating security controls:

```text
MITRE ATLAS Technique                 Compensating NIST SP 800-53 Controls
───────────────────────────────────   ───────────────────────────────────────
AML.T0051 (LLM Prompt Injection)  ──> SI-10 (Information Input Validation)
                                      AC-3  (Access Enforcement)
                                      SI-4  (Information System Monitoring)

AML.T0054 (LLM Jailbreak)         ──> SI-10 (Information Input Validation)
                                      SC-7  (Boundary Protection)
                                      SA-11 (Developer Testing & Evaluation)

AML.T0043 (Insecure Output)       ──> SI-10 (Information Input Validation)
                                      SC-8  (Transmission Integrity)

AML.T0018 (Data Poisoning)        ──> SA-12 (Supply Chain Risk Management)
                                      SI-7  (Information Integrity)

AML.T0044 (Model Extraction)      ──> SC-13 (Cryptographic Protection)
                                      AC-4  (Information Flow Enforcement)
```

---

## 5. Verification Commands

All artifacts can be verified locally using `mizan` or `oscal-cli`:

```bash
# Validate Anthropic component definition
mizan validate examples/usecases/anthropic-claude-component.json --strict

# Validate Google Vertex component definition
mizan validate examples/usecases/google-gemini-vertex-component.json --strict

# Validate OpenAI Azure component definition
mizan validate examples/usecases/openai-gpt4-azure-component.json --strict

# Validate Cohere North component definition
mizan validate examples/usecases/cohere-north-component.json --strict

# Validate MITRE ATLAS catalog
mizan validate examples/usecases/mitre-atlas-catalog.json --strict

# Validate NIST to MITRE crosswalk
mizan validate examples/usecases/nist-to-mitre-atlas-crosswalk.json --strict

# Validate Frontier AI profile
mizan validate examples/usecases/frontier-ai-mitre-profile.json --strict
```

# dart-transport (delta)

## ADDED Requirements

### Requirement: The Dart async-transport deferral MUST carry a dated upstream check

The Dart async-transport deferral MUST be recorded with a dated upstream check rather than
an open-ended "coming soon," so the deferral does not silently rot. The documentation MUST
name the date checked, the installed `uniffi-bindgen-dart` version, whether a newer
async-fixing release was found, and the explicit follow-up trigger (regenerate and remove
the post-generation patch when a fixed generator ships). No async transport code is written
while the generator remains broken.

#### Scenario: the deferral note is dated and actionable

- **WHEN** the Dart async-transport deferral is documented
- **THEN** it names the check date, the installed generator version, and the removal trigger
- **AND** no async transport method is claimed to work while the generator is unfixed

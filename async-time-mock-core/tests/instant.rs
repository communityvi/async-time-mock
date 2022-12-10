use async_time_mock_core::{Instant, TimerRegistry};

#[test]
#[should_panic]
fn should_not_allow_calculating_duration_between_instants_from_different_timer_registries() {
	let (instant1, instant2) = instants_from_different_timer_registries();
	instant2.duration_since(instant1);
}

#[test]
#[should_panic]
fn should_not_allow_calculating_checked_duration_between_instants_from_different_timer_registries() {
	let (instant1, instant2) = instants_from_different_timer_registries();
	instant2.checked_duration_since(instant1);
}

#[test]
#[should_panic]
fn should_not_allow_calculating_saturated_duration_between_instants_from_different_timer_registries() {
	let (instant1, instant2) = instants_from_different_timer_registries();
	instant2.saturated_duration_since(instant1);
}

#[test]
#[should_panic]
fn should_not_allow_subtracting_two_instants_from_different_timer_registries() {
	let (instant1, instant2) = instants_from_different_timer_registries();
	let _ = instant2 - instant1;
}

fn instants_from_different_timer_registries() -> (Instant, Instant) {
	let timer_registry1 = TimerRegistry::default();
	let timer_registry2 = TimerRegistry::default();

	(timer_registry1.now(), timer_registry2.now())
}

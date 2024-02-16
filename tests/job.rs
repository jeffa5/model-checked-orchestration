use common::run;
use common::LogicalBoolExt;
use stateright::Expectation;
use std::collections::BTreeMap;
use stdext::function_name;
use themelios::controller::client::ClientState;
use themelios::controller::job::JOB_TRACKING_FINALIZER;
use themelios::controller::util::is_pod_active;
use themelios::controller::util::is_pod_ready;
use themelios::model::OrchestrationModelCfg;
use themelios::resources::Container;
use themelios::resources::Job;
use themelios::resources::JobSpec;
use themelios::resources::Metadata;
use themelios::resources::PodPhase;
use themelios::resources::PodSpec;
use themelios::resources::PodTemplateSpec;
use themelios::state::RawState;
use themelios::utils;

mod common;

fn model(jobs: impl IntoIterator<Item = Job>, client_state: ClientState) -> OrchestrationModelCfg {
    let initial_state = RawState::default().with_jobs(jobs);
    let mut model = OrchestrationModelCfg {
        initial_state,
        job_controllers: 1,
        schedulers: 1,
        nodes: 1,
        client_state,
        ..Default::default()
    };
    model.add_property(
        Expectation::Always,
        "when synced, job status matches pods",
        |_model, s| {
            let s = s.latest();
            s.jobs.iter().all(|r| {
                let active_pods = s
                    .pods
                    .for_controller(&r.metadata.uid)
                    .filter(|p| is_pod_active(p))
                    .count();
                let ready_pods = s
                    .pods
                    .for_controller(&r.metadata.uid)
                    .filter(|p| is_pod_ready(p))
                    .count();
                // when the resource has finished processing towards the desired state the
                // status should match the desired number of replicas and the pods should match
                // that too
                let stable = s.resource_current(r);
                // mimic validateJobPodsStatus
                let active_correct = active_pods as u32 == r.status.active;
                let ready_correct = ready_pods as u32 == r.status.ready;
                stable.implies(active_correct && ready_correct)
            })
        },
    );
    model.add_property(
        Expectation::Always,
        "owned active pods have tracking finalizer",
        |_model, s| {
            let s = s.latest();
            s.jobs.iter().all(|r| {
                s.pods
                    .for_controller(&r.metadata.uid)
                    .filter(|p| is_pod_active(p))
                    .all(|p| {
                        p.metadata
                            .finalizers
                            .contains(&JOB_TRACKING_FINALIZER.to_string())
                    })
            })
        },
    );
    model.add_property(
        Expectation::Always,
        "finished pods have no finalizer",
        |_model, s| {
            let s = s.latest();
            s.jobs.iter().all(|r| {
                s.pods
                    .for_controller(&r.metadata.uid)
                    .filter(|p| matches!(p.status.phase, PodPhase::Succeeded | PodPhase::Failed))
                    .all(|p| {
                        !p.metadata
                            .finalizers
                            .contains(&JOB_TRACKING_FINALIZER.to_string())
                    })
            })
        },
    );
    model
}

fn new_job(name: &str, _namespace: &str) -> Job {
    let mut d = Job {
        metadata: utils::metadata(name.to_owned()),
        spec: JobSpec {
            ..Default::default()
        },
        ..Default::default()
    };
    let mut test_labels = BTreeMap::new();
    test_labels.insert("name".to_owned(), "test".to_owned());
    d.spec.selector.match_labels = test_labels.clone();
    d.spec.template = PodTemplateSpec {
        metadata: Metadata {
            labels: test_labels.clone(),
            ..Default::default()
        },
        spec: PodSpec {
            containers: vec![Container {
                name: "fake".to_owned(),
                image: "fake".to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        },
    };
    d
}

// func TestNonParallelJob(t *testing.T) {
#[test_log::test]
fn test_non_parallel_job() {
    let job = new_job("simple", "");

    let m = model([job], ClientState::default());
    run(m, common::CheckMode::Bfs, function_name!())
}

// func TestParallelJob(t *testing.T) {
#[test_log::test]
fn test_parallel_job() {
    let mut job = new_job("simple", "");

    job.spec.parallelism = 5;

    // TODO: have a way of failing pods and check that.

    let m = model([job], ClientState::default());
    run(m, common::CheckMode::Bfs, function_name!())
}

// TESTS TO DO
// func TestJobPodFailurePolicyWithFailedPodDeletedDuringControllerRestart(t *testing.T) {
// func TestJobPodFailurePolicy(t *testing.T) {
// func TestParallelJobParallelism(t *testing.T) {
// func TestParallelJobWithCompletions(t *testing.T) {
// func TestIndexedJob(t *testing.T) {
// func TestJobPodReplacementPolicy(t *testing.T) {
// func TestElasticIndexedJob(t *testing.T) {
// func TestOrphanPodsFinalizersClearedWithGC(t *testing.T) {
// func TestJobFailedWithInterrupts(t *testing.T) {
// func TestOrphanPodsFinalizersClearedOnRestart(t *testing.T) {
// func TestSuspendJob(t *testing.T) {
// func TestSuspendJobControllerRestart(t *testing.T) {
// func TestNodeSelectorUpdate(t *testing.T) {

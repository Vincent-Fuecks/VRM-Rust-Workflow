use std::collections::HashMap;

use crate::{
    api::rms_config_dto::rms_dto::SlurmRmsDto,
    domain::vrm_system_model::{
        rms::slurm::{response::slurm_node::SlurmNodesResponse, slurm::SlurmRms},
        schedule::slotted_schedule::network_slotted_schedule::topology::{Link, Node},
        utils::id::{NodeResourceId, RouterId},
    },
};

impl SlurmRms {
    pub fn get_nodes_and_links(dto: &SlurmRmsDto, nodes_response: &SlurmNodesResponse) -> (Vec<Node>, Vec<Link>) {
        let mut links = Vec::new();
        let mut nodes = Vec::new();
        let mut node_to_switches: HashMap<NodeResourceId, Vec<RouterId>> = HashMap::new();

        for start_switch in &dto.topology {
            for end_switch in &start_switch.switches {
                let link = Link {
                    id: RouterId::new(start_switch.switch_name.clone()),
                    source: RouterId::new(start_switch.switch_name.clone()),
                    target: RouterId::new(end_switch.clone()),
                    capacity: start_switch.link_speed,
                };

                let node_ids: Vec<NodeResourceId> = start_switch.nodes.iter().map(|node_id| NodeResourceId::new(node_id)).collect();

                for node_id in node_ids {
                    node_to_switches.entry(node_id).or_insert_with(Vec::new).push(link.id.clone());
                }

                links.push(link);
            }
        }

        for slurm_node in &nodes_response.nodes {
            let node_id = NodeResourceId::new(slurm_node.name.clone());

            if node_to_switches.contains_key(&node_id) {
                let node =
                    Node { id: node_id.clone(), cpus: slurm_node.cpus as i64, connected_to_router: node_to_switches.get(&node_id).unwrap().clone() };

                nodes.push(node);
            } else if !node_to_switches.is_empty() {
                log::error!(
                    "SlurmNetworkConstructionError: The compute node {} of cluster {} was not found in the topology. Please check your submitted topology.",
                    slurm_node.name,
                    slurm_node.cluster_name
                );
            }
        }
        return (nodes, links);
    }
}

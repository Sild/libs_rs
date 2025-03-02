#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    Library,
    MerkleProof,
    MerkleUpdate,
}

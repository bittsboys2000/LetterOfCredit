// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract LCRegistry {
    // Event to listen for off-chain
    event StateUpdated(string indexed lcId, string state, string documentHash, address indexed actor, uint256 timestamp);

    struct LCState {
        string currentState;
        string documentHash;
        address lastActor;
        uint256 timestamp;
    }

    // Mapping from LC ID (UUID string) to its latest state
    mapping(string => LCState) public lcStates;

    // Access Control (Simple version: anyone can call, in prod use RBAC)
    function updateLCState(string memory _lcId, string memory _state, string memory _docHash) public {
        lcStates[_lcId] = LCState({
            currentState: _state,
            documentHash: _docHash,
            lastActor: msg.sender,
            timestamp: block.timestamp
        });

        emit StateUpdated(_lcId, _state, _docHash, msg.sender, block.timestamp);
    }

    function getLCState(string memory _lcId) public view returns (string memory, string memory, address, uint256) {
        LCState memory s = lcStates[_lcId];
        return (s.currentState, s.documentHash, s.lastActor, s.timestamp);
    }
}

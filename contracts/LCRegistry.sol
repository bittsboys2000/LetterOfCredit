// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title LCRegistry
 * @dev Letter of Credit state registry with role-based access control
 */
contract LCRegistry {
    // ============ Events ============
    event StateUpdated(string indexed lcId, string state, string documentHash, address indexed actor, uint256 timestamp);
    event OperatorAdded(address indexed operator, string role);
    event OperatorRemoved(address indexed operator);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    // ============ Structs ============
    struct LCState {
        string currentState;
        string documentHash;
        address lastActor;
        uint256 timestamp;
        bool exists;
    }

    struct Operator {
        bool isActive;
        string role; // "IssuingBank", "AdvisingBank", "ChainAdapter"
        uint256 addedAt;
    }

    // ============ State Variables ============
    address public owner;
    mapping(string => LCState) public lcStates;
    mapping(address => Operator) public operators;
    address[] public operatorList;
    
    // Valid state transitions
    mapping(string => mapping(string => bool)) public validTransitions;

    // ============ Modifiers ============
    modifier onlyOwner() {
        require(msg.sender == owner, "LCRegistry: caller is not the owner");
        _;
    }

    modifier onlyAuthorized() {
        require(
            msg.sender == owner || operators[msg.sender].isActive,
            "LCRegistry: caller is not authorized"
        );
        _;
    }

    // ============ Constructor ============
    constructor() {
        owner = msg.sender;
        _initializeTransitions();
    }

    function _initializeTransitions() private {
        // Define valid state transitions
        validTransitions[""]["Draft"] = true;              // Initial creation
        validTransitions["Draft"]["Submitted"] = true;
        validTransitions["Submitted"]["Review"] = true;
        validTransitions["Review"]["Approved"] = true;
        validTransitions["Review"]["Rejected"] = true;
        validTransitions["Approved"]["Closed"] = true;
    }

    // ============ Owner Functions ============
    
    /**
     * @dev Transfer ownership to a new address
     */
    function transferOwnership(address newOwner) public onlyOwner {
        require(newOwner != address(0), "LCRegistry: new owner is zero address");
        emit OwnershipTransferred(owner, newOwner);
        owner = newOwner;
    }

    /**
     * @dev Add an authorized operator (e.g., bank addresses, chain adapter service)
     */
    function addOperator(address _operator, string memory _role) public onlyOwner {
        require(_operator != address(0), "LCRegistry: operator is zero address");
        require(!operators[_operator].isActive, "LCRegistry: operator already exists");
        
        operators[_operator] = Operator({
            isActive: true,
            role: _role,
            addedAt: block.timestamp
        });
        operatorList.push(_operator);
        
        emit OperatorAdded(_operator, _role);
    }

    /**
     * @dev Remove an authorized operator
     */
    function removeOperator(address _operator) public onlyOwner {
        require(operators[_operator].isActive, "LCRegistry: operator not found");
        
        operators[_operator].isActive = false;
        emit OperatorRemoved(_operator);
    }

    // ============ Core Functions ============

    /**
     * @dev Update LC state with validation
     * @param _lcId The Letter of Credit ID (UUID)
     * @param _state The new state
     * @param _docHash The document hash (IPFS CID or SHA-256)
     */
    function updateLCState(
        string memory _lcId, 
        string memory _state, 
        string memory _docHash
    ) public onlyAuthorized {
        LCState storage current = lcStates[_lcId];
        string memory currentState = current.exists ? current.currentState : "";
        
        // Validate state transition
        require(
            validTransitions[currentState][_state],
            string(abi.encodePacked("LCRegistry: invalid transition from '", currentState, "' to '", _state, "'"))
        );

        lcStates[_lcId] = LCState({
            currentState: _state,
            documentHash: _docHash,
            lastActor: msg.sender,
            timestamp: block.timestamp,
            exists: true
        });

        emit StateUpdated(_lcId, _state, _docHash, msg.sender, block.timestamp);
    }

    /**
     * @dev Force update LC state (owner only, for emergency corrections)
     */
    function forceUpdateLCState(
        string memory _lcId, 
        string memory _state, 
        string memory _docHash
    ) public onlyOwner {
        lcStates[_lcId] = LCState({
            currentState: _state,
            documentHash: _docHash,
            lastActor: msg.sender,
            timestamp: block.timestamp,
            exists: true
        });

        emit StateUpdated(_lcId, _state, _docHash, msg.sender, block.timestamp);
    }

    // ============ View Functions ============

    /**
     * @dev Get LC state
     */
    function getLCState(string memory _lcId) public view returns (
        string memory currentState,
        string memory documentHash,
        address lastActor,
        uint256 timestamp,
        bool exists
    ) {
        LCState memory s = lcStates[_lcId];
        return (s.currentState, s.documentHash, s.lastActor, s.timestamp, s.exists);
    }

    /**
     * @dev Check if an address is an authorized operator
     */
    function isOperator(address _addr) public view returns (bool) {
        return operators[_addr].isActive;
    }

    /**
     * @dev Get operator details
     */
    function getOperator(address _addr) public view returns (bool isActive, string memory role, uint256 addedAt) {
        Operator memory op = operators[_addr];
        return (op.isActive, op.role, op.addedAt);
    }

    /**
     * @dev Get total number of operators
     */
    function getOperatorCount() public view returns (uint256) {
        uint256 count = 0;
        for (uint256 i = 0; i < operatorList.length; i++) {
            if (operators[operatorList[i]].isActive) {
                count++;
            }
        }
        return count;
    }

    /**
     * @dev Check if a state transition is valid
     */
    function isValidTransition(string memory _from, string memory _to) public view returns (bool) {
        return validTransitions[_from][_to];
    }
}

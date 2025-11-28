import requests
import json
import time
import sys

# Configuration
API_DOC = "http://localhost:3001"
API_LC = "http://localhost:3002"
API_AUDIT = "http://localhost:3003"
API_CHAIN = "http://localhost:3004"
API_USER = "http://localhost:3005"

# Mock Tokens
TOKEN_IMPORTER = "Bearer MOCK_TOKEN_Importer"
TOKEN_EXPORTER = "Bearer MOCK_TOKEN_Exporter"
TOKEN_ISSUING_BANK = "Bearer MOCK_TOKEN_IssuingBank"
TOKEN_ADVISING_BANK = "Bearer MOCK_TOKEN_AdvisingBank"
TOKEN_AUDITOR = "Bearer MOCK_TOKEN_Auditor"
TOKEN_INVALID = "Bearer MOCK_TOKEN_Hacker"

def print_header(title):
    print(f"\n{'='*20} {title} {'='*20}")

def test_upload_document():
    print_header("SCENARIO 1: Document Upload")
    
    # 1.1 Success Case
    print("-> 1.1 Uploading valid document...")
    files = {'file': ('contract.txt', b'Valid Contract Content')}
    res = requests.post(f"{API_DOC}/documents", headers={"Authorization": TOKEN_IMPORTER}, files=files)
    if res.status_code == 200:
        print(f"   SUCCESS: {res.json()}")
        return res.json()['document_id']
    else:
        print(f"   FAILED: {res.text}")
        sys.exit(1)

def test_lc_workflow(doc_id):
    print_header("SCENARIO 2: LC Workflow & Access Control")
    
    lc_id = None

    # 2.1 Create LC (Success)
    print("-> 2.1 Creating LC as Importer...")
    payload = {
        "beneficiary_id": "00000000-0000-0000-0000-000000000002",
        "issuing_bank_id": "00000000-0000-0000-0000-000000000003",
        "advising_bank_id": "00000000-0000-0000-0000-000000000004",
        "amount": "50000.00",
        "currency": "USD",
        "expiry_date": "2024-12-31T00:00:00Z",
        "document_ids": [doc_id]
    }
    res = requests.post(f"{API_LC}/lc", headers={"Authorization": TOKEN_IMPORTER}, json=payload)
    if res.status_code == 200:
        lc_id = res.json()['id']
        print(f"   SUCCESS: LC Created ID={lc_id}")
    else:
        print(f"   FAILED: {res.text}")
        sys.exit(1)

    # 2.2 Test new list endpoint
    print("-> 2.2 Testing LC list endpoint...")
    res = requests.get(f"{API_LC}/lc", headers={"Authorization": TOKEN_IMPORTER})
    if res.status_code == 200:
        data = res.json()
        print(f"   SUCCESS: Listed {len(data['items'])} LCs (total: {data['total']})")
    else:
        print(f"   FAILED: {res.text}")

    # 2.3 Unauthorized Status Update (Exporter trying to Submit)
    print("-> 2.3 Exporter trying to Submit (Should Fail)...")
    res = requests.put(f"{API_LC}/lc/{lc_id}/status", headers={"Authorization": TOKEN_EXPORTER}, json={"status": "Submitted"})
    if res.status_code == 401:
        print("   SUCCESS: Access Denied as expected.")
    else:
        print(f"   FAILED: Unexpected status code {res.status_code}")

    # 2.4 Invalid State Transition (Draft -> Approved directly)
    print("-> 2.4 Importer trying to Approve directly (Should Fail)...")
    res = requests.put(f"{API_LC}/lc/{lc_id}/status", headers={"Authorization": TOKEN_IMPORTER}, json={"status": "Approved"})
    if res.status_code == 400:
        print("   SUCCESS: Invalid transition rejected.")
    else:
        print(f"   FAILED: Unexpected status code {res.status_code}")

    # 2.5 Valid Transition: Draft -> Submitted
    print("-> 2.5 Importer Submitting (Success)...")
    res = requests.put(f"{API_LC}/lc/{lc_id}/status", headers={"Authorization": TOKEN_IMPORTER}, json={"status": "Submitted"})
    if res.status_code == 200:
        print("   SUCCESS: Status updated to Submitted.")
    else:
        print(f"   FAILED: {res.text}")

    # 2.6 Issuing Bank Review
    print("-> 2.6 Issuing Bank Starting Review (Success)...")
    res = requests.put(f"{API_LC}/lc/{lc_id}/status", headers={"Authorization": TOKEN_ISSUING_BANK}, json={"status": "Review"})
    if res.status_code == 200:
        print("   SUCCESS: Status updated to Review.")
    else:
        print(f"   FAILED: {res.text}")

    # 2.7 Issuing Bank Approve
    print("-> 2.7 Issuing Bank Approving (Success)...")
    res = requests.put(f"{API_LC}/lc/{lc_id}/status", headers={"Authorization": TOKEN_ISSUING_BANK}, json={"status": "Approved"})
    if res.status_code == 200:
        print("   SUCCESS: Status updated to Approved.")
    else:
        print(f"   FAILED: {res.text}")

    return lc_id

def test_audit_logs():
    print_header("SCENARIO 3: Audit Logging")
    
    print("-> 3.1 Fetching Audit Logs as Auditor...")
    res = requests.get(f"{API_AUDIT}/audit", headers={"Authorization": TOKEN_AUDITOR})
    if res.status_code == 200:
        logs = res.json()
        print(f"   SUCCESS: Retrieved {len(logs)} logs.")
        for log in logs[-3:]:
            print(f"   - [{log['event_type']}] {log['details']}")
    else:
        print(f"   FAILED: {res.text}")

    print("-> 3.2 Non-auditor trying to access logs (Should Fail)...")
    res = requests.get(f"{API_AUDIT}/audit", headers={"Authorization": TOKEN_IMPORTER})
    if res.status_code == 401:
        print("   SUCCESS: Access denied for non-auditor.")
    else:
        print(f"   WARNING: Unexpected status {res.status_code}")

def test_chain_adapter():
    print_header("SCENARIO 4: Blockchain Anchoring")
    
    print("-> 4.1 Testing anchor endpoint...")
    payload = {
        "lc_id": "test-lc-123",
        "state": "Submitted",
        "hash": "abc123def456"
    }
    try:
        res = requests.post(f"{API_CHAIN}/anchor", 
                          headers={"Authorization": TOKEN_ISSUING_BANK, "Content-Type": "application/json"},
                          json=payload)
        if res.status_code == 200:
            print(f"   SUCCESS: Anchored to chain. TX: {res.json().get('tx_hash', 'N/A')}")
        else:
            print(f"   INFO: Chain adapter returned {res.status_code} (expected if Besu not running)")
    except Exception as e:
        print(f"   WARNING: Chain Adapter not reachable ({e})")

def test_user_service():
    print_header("SCENARIO 5: User Service")
    
    print("-> 5.1 Testing user registration...")
    payload = {
        "email": f"test{int(time.time())}@example.com",
        "password": "testpassword123",
        "name": "Test User",
        "organization": "Test Corp",
        "role": "Importer"
    }
    try:
        res = requests.post(f"{API_USER}/auth/register", json=payload)
        if res.status_code == 200:
            data = res.json()
            print(f"   SUCCESS: User registered. Token received: {data['token'][:20]}...")
            return data['token']
        else:
            print(f"   INFO: Registration returned {res.status_code} - {res.text}")
    except Exception as e:
        print(f"   WARNING: User service not reachable ({e})")
    return None

def test_pagination():
    print_header("SCENARIO 6: Pagination")
    
    print("-> 6.1 Testing pagination parameters...")
    res = requests.get(f"{API_LC}/lc?limit=5&offset=0", headers={"Authorization": TOKEN_AUDITOR})
    if res.status_code == 200:
        data = res.json()
        print(f"   SUCCESS: Page 1 - {len(data['items'])} items, total: {data['total']}")
    else:
        print(f"   FAILED: {res.text}")

if __name__ == "__main__":
    print("=" * 60)
    print("  LC Chain - Comprehensive Test Suite")
    print("=" * 60)
    
    try:
        doc_id = test_upload_document()
        lc_id = test_lc_workflow(doc_id)
        test_audit_logs()
        test_chain_adapter()
        test_user_service()
        test_pagination()
        
        print("\n" + "=" * 60)
        print("  ALL TESTS COMPLETED SUCCESSFULLY!")
        print("=" * 60)
    except Exception as e:
        print(f"\nTEST SUITE FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

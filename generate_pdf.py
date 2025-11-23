import sys

def create_pdf(filename):
    objects = []
    
    # 1. Catalog
    objects.append(b"<< /Type /Catalog /Pages 2 0 R >>")
    
    # 2. Pages
    objects.append(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    
    # 3. Page
    objects.append(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>")
    
    # 4. Content Stream
    stream = b"BT /F1 24 Tf 100 700 Td (TEST CONTRACT PDF) Tj ET"
    objects.append(b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"\nendstream")
    
    # 5. Font
    objects.append(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")

    with open(filename, 'wb') as f:
        f.write(b"%PDF-1.1\n")
        offsets = []
        
        for i, obj in enumerate(objects):
            offsets.append(f.tell())
            f.write(f"{i+1} 0 obj\n".encode())
            f.write(obj)
            f.write(b"\nendobj\n")
            
        xref_start = f.tell()
        f.write(b"xref\n")
        f.write(f"0 {len(objects)+1}\n".encode())
        f.write(b"0000000000 65535 f \n")
        for offset in offsets:
            f.write(f"{offset:010d} 00000 n \n".encode())
            
        f.write(b"trailer\n")
        f.write(f"<< /Size {len(objects)+1} /Root 1 0 R >>\n".encode())
        f.write(b"startxref\n")
        f.write(f"{xref_start}\n".encode())
        f.write(b"%%EOF\n")

if __name__ == "__main__":
    create_pdf("test_contract.pdf")
    print("PDF Created")

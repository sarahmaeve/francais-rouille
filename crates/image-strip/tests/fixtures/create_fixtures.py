#!/usr/bin/env python3
"""Generate minimal JPEG and PNG test fixtures with EXIF/metadata."""
import struct
import zlib

def create_jpeg_with_exif():
    """Create a minimal 1x1 JPEG with an APP1/EXIF segment and a COM segment."""
    # SOI
    data = b'\xFF\xD8'

    # APP0 (JFIF) - minimal
    app0_payload = b'JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00'
    data += b'\xFF\xE0' + struct.pack('>H', len(app0_payload) + 2) + app0_payload

    # APP1 (fake EXIF with GPS-like data)
    exif_payload = b'Exif\x00\x00' + b'FakeGPSData:48.8566,2.3522' + b'\x00' * 20
    data += b'\xFF\xE1' + struct.pack('>H', len(exif_payload) + 2) + exif_payload

    # APP13 (fake IPTC)
    iptc_payload = b'Photoshop 3.0\x00' + b'FakeIPTC:photographer=John' + b'\x00' * 10
    data += b'\xFF\xED' + struct.pack('>H', len(iptc_payload) + 2) + iptc_payload

    # COM (comment)
    comment = b'Secret comment with device info'
    data += b'\xFF\xFE' + struct.pack('>H', len(comment) + 2) + comment

    # DQT - minimal quantization table
    dqt = bytes([0] + [1]*64)
    data += b'\xFF\xDB' + struct.pack('>H', len(dqt) + 2) + dqt

    # SOF0 - 1x1 pixel, 1 component
    sof = struct.pack('>BHHB', 8, 1, 1, 1) + b'\x01\x11\x00'
    data += b'\xFF\xC0' + struct.pack('>H', len(sof) + 2) + sof

    # DHT - minimal Huffman table
    dht = b'\x00' + bytes(16) + b'\x00'
    data += b'\xFF\xC4' + struct.pack('>H', len(dht) + 2) + dht

    # SOS
    sos = struct.pack('>B', 1) + b'\x01\x00' + b'\x00\x3F\x00'
    data += b'\xFF\xDA' + struct.pack('>H', len(sos) + 2) + sos

    # Minimal scan data (one byte)
    data += b'\x00'

    # EOI
    data += b'\xFF\xD9'

    return data


def create_jpeg_with_icc():
    """Create a minimal JPEG that also has an APP2/ICC segment."""
    data = create_jpeg_with_exif()
    # Insert APP2 (fake ICC) right after SOI + APP0, before APP1
    # Find APP1 position
    app1_pos = data.index(b'\xFF\xE1')
    icc_payload = b'ICC_PROFILE\x00\x01\x01' + b'\x00' * 20
    app2 = b'\xFF\xE2' + struct.pack('>H', len(icc_payload) + 2) + icc_payload
    data = data[:app1_pos] + app2 + data[app1_pos:]
    return data


def create_png_with_text():
    """Create a minimal 1x1 PNG with tEXt and eXIf chunks."""
    # PNG signature
    sig = b'\x89PNG\r\n\x1a\n'

    def make_chunk(chunk_type, chunk_data):
        c = chunk_type + chunk_data
        crc = struct.pack('>I', zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack('>I', len(chunk_data)) + c + crc

    # IHDR: 1x1, 8-bit grayscale
    ihdr_data = struct.pack('>IIBBBBB', 1, 1, 8, 0, 0, 0, 0)
    ihdr = make_chunk(b'IHDR', ihdr_data)

    # tEXt chunk: Author=Secret Person
    text_data = b'Author\x00Secret Person'
    text_chunk = make_chunk(b'tEXt', text_data)

    # eXIf chunk: fake EXIF
    exif_data = b'Exif\x00\x00FakeGPS:48.8566,2.3522'
    exif_chunk = make_chunk(b'eXIf', exif_data)

    # tIME chunk
    time_data = struct.pack('>HBBBBB', 2025, 6, 15, 10, 30, 0)
    time_chunk = make_chunk(b'tIME', time_data)

    # IDAT: compressed scanline (filter=0, pixel=0)
    raw = b'\x00\x00'  # filter byte + 1 grayscale byte
    compressed = zlib.compress(raw)
    idat = make_chunk(b'IDAT', compressed)

    # IEND
    iend = make_chunk(b'IEND', b'')

    return sig + ihdr + text_chunk + exif_chunk + time_chunk + idat + iend


def create_png_with_iccp():
    """Create a minimal PNG that also has an iCCP chunk."""
    sig = b'\x89PNG\r\n\x1a\n'

    def make_chunk(chunk_type, chunk_data):
        c = chunk_type + chunk_data
        crc = struct.pack('>I', zlib.crc32(c) & 0xFFFFFFFF)
        return struct.pack('>I', len(chunk_data)) + c + crc

    ihdr_data = struct.pack('>IIBBBBB', 1, 1, 8, 0, 0, 0, 0)
    ihdr = make_chunk(b'IHDR', ihdr_data)

    # iCCP chunk
    iccp_data = b'sRGB\x00\x00' + zlib.compress(b'\x00' * 20)
    iccp_chunk = make_chunk(b'iCCP', iccp_data)

    text_data = b'Comment\x00Remove me'
    text_chunk = make_chunk(b'tEXt', text_data)

    raw = b'\x00\x00'
    compressed = zlib.compress(raw)
    idat = make_chunk(b'IDAT', compressed)
    iend = make_chunk(b'IEND', b'')

    return sig + ihdr + iccp_chunk + text_chunk + idat + iend


if __name__ == '__main__':
    import os
    d = os.path.dirname(os.path.abspath(__file__))

    with open(os.path.join(d, 'with_exif.jpg'), 'wb') as f:
        f.write(create_jpeg_with_exif())
    with open(os.path.join(d, 'with_icc.jpg'), 'wb') as f:
        f.write(create_jpeg_with_icc())
    with open(os.path.join(d, 'with_text.png'), 'wb') as f:
        f.write(create_png_with_text())
    with open(os.path.join(d, 'with_iccp.png'), 'wb') as f:
        f.write(create_png_with_iccp())

    print('Fixtures created.')

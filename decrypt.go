package main

func DecodeIndex(mangaID, epID int64, data []byte) []byte {
	key := []byte{
		byte(epID & 0xff), byte(epID >> 8 & 0xff), byte(epID >> 16 & 0xff), byte(epID >> 24 & 0xff),
		byte(mangaID & 0xff), byte(mangaID >> 8 & 0xff), byte(mangaID >> 16 & 0xff), byte(mangaID >> 24 & 0xff),
	}

	for i := 0; i < len(data); i++ {
		data[i] ^= key[i%8]
	}

	return data
}

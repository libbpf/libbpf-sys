#include "libbpf/include/uapi/linux/if_link.h"
#include "libbpf/src/bpf.h"
#include "libbpf/src/btf.h"
#include "libbpf/src/libbpf.h"
#include "libbpf/src/xsk.h"

extern __u64 *_xsk_ring_prod__fill_addr(struct xsk_ring_prod *fill, __u32 idx);

extern const __u64 *_xsk_ring_cons__comp_addr(const struct xsk_ring_cons *comp, __u32 idx);

extern size_t _xsk_ring_cons__peek(struct xsk_ring_cons *cons, size_t nb, __u32 *idx);

extern void _xsk_ring_cons__release(struct xsk_ring_cons *cons, size_t nb);

extern size_t _xsk_ring_prod__reserve(struct xsk_ring_prod *prod, size_t nb, __u32 *idx);

extern void _xsk_ring_prod__submit(struct xsk_ring_prod *prod, size_t nb);

extern const struct xdp_desc *_xsk_ring_cons__rx_desc(const struct xsk_ring_cons *rx, __u32 idx);

extern struct xdp_desc *_xsk_ring_prod__tx_desc(struct xsk_ring_prod *tx, __u32 idx);

extern void *_xsk_umem__get_data(void *umem_area, __u64 addr);

extern int _xsk_ring_prod__needs_wakeup(const struct xsk_ring_prod *r);

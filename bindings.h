#ifdef __LIBBPF_SYS_NOVENDOR
#include <linux/if_link.h>
#include <linux/perf_event.h>
#include <bpf/bpf.h>
#include <bpf/btf.h>
#include <bpf/libbpf.h>
#include <bpf/xsk.h>
#else
#include "libbpf/include/uapi/linux/if_link.h"
#include "libbpf/include/uapi/linux/perf_event.h"
#include "libbpf/src/bpf.h"
#include "libbpf/src/btf.h"
#include "libbpf/src/libbpf.h"
#include "libbpf/src/xsk.h"
#endif /* __LIBBPF_SYS_NOVENDOR */

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

extern int _xsk_prod_nb_free(struct xsk_ring_prod *r, __u32 nb);

extern int _xsk_cons_nb_avail(struct xsk_ring_cons *r, __u32 nb);

extern __u64 _xsk_umem__extract_addr(__u64 addr);

extern __u64 _xsk_umem__extract_offset(__u64 addr);

extern __u64 _xsk_umem__add_offset_to_addr(__u64 addr);

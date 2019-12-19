#include "bindings.h"

//
// Wrap the inline functions in libbpf with C functions so Rust bindings can be generated
//

inline __u64 *_xsk_ring_prod__fill_addr(struct xsk_ring_prod *fill, __u32 idx)
{
    return xsk_ring_prod__fill_addr(fill, idx);
}

inline const __u64 * _xsk_ring_cons__comp_addr(const struct xsk_ring_cons *comp, __u32 idx)
{
        return xsk_ring_cons__comp_addr(comp, idx);
}

inline size_t _xsk_ring_cons__peek(struct xsk_ring_cons *cons,
                            size_t nb, __u32 *idx)
{
    return xsk_ring_cons__peek(cons, nb, idx);
}

inline void _xsk_ring_cons__release(struct xsk_ring_cons *cons, size_t nb)
{
    xsk_ring_cons__release(cons, nb);
}

inline size_t _xsk_ring_prod__reserve(struct xsk_ring_prod *prod, size_t nb, __u32 *idx)
{
    return xsk_ring_prod__reserve(prod, nb, idx);
}

inline void _xsk_ring_prod__submit(struct xsk_ring_prod *prod, size_t nb)
{
    xsk_ring_prod__submit(prod, nb);
}

inline const struct xdp_desc *_xsk_ring_cons__rx_desc(const struct xsk_ring_cons *rx, __u32 idx)
{
    return xsk_ring_cons__rx_desc(rx, idx);
}

inline extern struct xdp_desc *_xsk_ring_prod__tx_desc(struct xsk_ring_prod *tx, __u32 idx) {
        return xsk_ring_prod__tx_desc(tx, idx);
}

inline void *_xsk_umem__get_data(void *umem_area, __u64 addr)
{
    return xsk_umem__get_data(umem_area, addr);
}

inline int _xsk_ring_prod__needs_wakeup(const struct xsk_ring_prod *r) {
        return xsk_ring_prod__needs_wakeup(r);
}

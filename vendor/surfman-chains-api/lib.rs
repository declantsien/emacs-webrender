/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// The consumer's view of a swap chain
pub trait SwapChainAPI: 'static + Clone + Send {
    type Surface;

    /// Take the current front buffer.
    fn take_surface(&self) -> Option<Self::Surface>;

    /// Recycle the current front buffer.
    fn recycle_surface(&self, surface: Self::Surface);
}

/// The consumer's view of a collection of swap chains
pub trait SwapChainsAPI<SwapChainID>: 'static + Clone + Send {
    type Surface;
    type SwapChain: SwapChainAPI<Surface = Self::Surface>;

    /// Get a swap chain
    fn get(&self, id: SwapChainID) -> Option<Self::SwapChain>;
}

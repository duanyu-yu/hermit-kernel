searchState.loadedDescShard("fdt", 0, "<code>fdt</code>\nThe FDT had an invalid magic value\nThe given pointer was null\nThe slice passed in was too small to fit the given total …\nA flattened devicetree located somewhere in memory\nPossible errors when attempting to create an <code>Fdt</code>\nReturn the <code>/aliases</code> node, if one exists\nReturns an iterator over all of the nodes in the …\nSearches for the <code>/chosen</code> node, which is always available\nReturn the <code>/cpus</code> node, which is always available\nReturns an iterator over all of the available nodes with …\nSearches for a node which contains a <code>compatible</code> property …\nReturns the first node that matches the node path, if you …\nSearches for the given <code>phandle</code>\nReturns the argument unchanged.\nReturns the argument unchanged.\nSafety\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the memory node, which is always available\nReturns an iterator over the memory reservations\nConstruct a new <code>Fdt</code> from a byte buffer\nReturn the root (<code>/</code>) node, which is always available\nReturns an iterator over all of the strings inside of the …\nTotal size of the devicetree in bytes\nThe number of cells (big endian u32s) that addresses and …\nA devicetree node\nA memory reservation\nA node property\nA raw <code>reg</code> property value set\nPointer representing the memory reservation address\nBig-endian encoded bytes making up the address portion of …\nSize of values representing an address\nAttempt to parse the property value as a <code>&amp;str</code>\nAttempt to parse the property value as a <code>usize</code>\nCell sizes for child nodes\nReturns an iterator over the children of the current node\n<code>compatible</code> property\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\n<code>#interrupt-cells</code> property\nSearches for the interrupt parent, if the node contains one\n<code>interrupts</code> property\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nProperty name\nReturns an iterator over the available properties of the …\nAttempts to find the a property by its name\nConvenience method that provides an iterator over the raw …\n<code>reg</code> property\nSize of the memory reservation\nBig-endian encoded bytes making up the size portion of the …\nSize of values representing a size\nProperty value\nRepresents the <code>/aliases</code> node with specific helper methods\nRepresents the <code>/chosen</code> node with specific helper methods\nRepresents the <code>compatible</code> property of a node\nRepresents a <code>/cpus/cpu*</code> node with specific helper methods\nRepresents the value of the <code>reg</code> property of a <code>/cpus/cpu*</code> …\nAn area described by the <code>initial-mapped-area</code> property of …\nRepresents the <code>/memory</code> node with specific helper methods\nA memory region\nRepresents the root (<code>/</code>) node with specific helper methods\nReturns an iterator over all of the available aliases\nReturns an iterator over all of the listed CPU IDs\nReturns an iterator over all available compatible strings\nContains the bootargs, if they exist\nRoot node cell sizes\n<code>clock-frequency</code> property\n<code>compatible</code> property\nEffective address of the mapped area\nThe first listed CPU ID, which will always exist\nFirst compatible string\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturn the IDs for the given CPU\nReturns the initial mapped area, if it exists\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\n<code>model</code> property\nPhysical address of the mapped area\nReturns an iterator over all of the available properties\nReturns an iterator over all of the properties for the CPU …\nAttempts to find the a property by its name\nAttempts to find the a property by its name\nReturns an iterator over all of the available memory …\nAttempt to resolve an alias to a node name\nAttempt to find the node specified by the given alias\nSize of the mapped area\nSize of the memory region\nStarting address represented as a pointer\nSearches for the node representing <code>stdout</code>, if the property …\nSearches for the node representing <code>stdout</code>, if the property …\n<code>timebase-frequency</code> property")
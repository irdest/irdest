include $(TOPDIR)/rules.mk

# Name, version and release number
# The name and version of your package are used to
# define the variable to point to the build directory
# of your package: $(PKG_BUILD_DIR)
PKG_NAME:=ratmand
PKG_VERSION:=0.5.0
PKG_RELEASE:=1
PKG_REV:=8c8c87fb62a4f4476b18c14f2d22fe164d097383
PKG_LICENSE:=AGPL-3.0
PKG_SOURCE_URL:=https://git.irde.st/we/irdest.git
PKG_SOURCE_PROTO:=git

PKG_SOURCE_SUBDIR:=$(PKG_NAME)-$(PKG_VERSION)
PKG_BUILD_DIR:=$(BUILD_DIR)/$(PKG_SOURCE_SUBDIR)
PKG_SOURCE:=$(PKG_NAME)-$(PKG_VERSION)
# https://git.irde.st/we/irdest/-/archive/release/ratmand-0.5.0/irdest-release-ratmand-0.5.0.tar.gz

include $(INCLUDE_DIR)/package.mk

# Package definition; instructs on how and where our
# package will appear in the overall configuration menu
# ('make menuconfig')
define Package/ratmand
	URL:=https://irde.st/
	SECTION:=net
	CATEGORY:=Network
	SUBMENU:=Routing and Redirection
	DEPENDS:=+libpthread +kmod-tun
	TITLE:=RATMAN Research project for decentralised communication
endef

#block-mount e2fsprogs kmod-fs-ext4 kmod-usb-storage kmod-usb2 kmod-usb3

# Package description; a more verbose description on
# what our package does
define Package/ratmand/description
	RATMAN Research project for decentralised communication
endef

# Package preparation instructions; create the build
# directory and copy the source code. The last command is
# necessary to ensure our preparation instructions remain
# compatible with the patching system.
define Build/Prepare
	mkdir -p $(PKG_BUILD_DIR)
	cp $(SOURCE_DIR)/* $(PKG_BUILD_DIR)
	$(Build/Patch)
endef

# Package build instructions; invoke the target-specific
# compiler to first compile the source file, and then
# to link the file into the final executable
define Build/Compile
	
endef

# Package install instructions; create a directory inside
# the package to hold our executable, and then copy the
# executable we built previously into the folder
define Package/ratmand/install
	
endef

# This command is always the last, it uses the definitions
# and variables we give above in order to get the job done
$(eval $(call BuildPackage,helloworld))

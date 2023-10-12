include $(TOPDIR)/rules.mk

# Name, version and release number
# The name and version of your package are used to
# define the variable to point to the build directory
# of your package: $(PKG_BUILD_DIR)
PKG_NAME:=ratmand
PKG_VERSION:=0.5.0
PKG_RELEASE:=1

PKG_SOURCE_PROTO:=git
PKG_SOURCE_URL:=https://git.irde.st/we/irdest.git
PKG_REV:=d6c030386c73ad22e75dfba0330c5c064ad1f234
PKG_LICENSE:=AGPL-3.0

# Source settings (i.e. where to find the source codes)
# This is a custom variable, used below
SOURCE_DIR:=/home/aj/proj/irdest/irdest

include $(INCLUDE_DIR)/package.mk


# Package definition; instructs on how and where our
# package will appear in the overall configuration menu
# ('make menuconfig')
define Package/ratmand
	URL:=https://irde.st/
	SECTION:=net
	CATEGORY:=Network
	TITLE:=RATMAN Research project for decentralised communication
endef

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

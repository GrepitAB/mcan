#[doc = "Register `RXFS` reader"]
pub struct R(crate::R<RXFS_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<RXFS_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<RXFS_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<RXFS_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `FFL` reader - Rx FIFO Fill Level"]
pub struct FFL_R(crate::FieldReader<u8, u8>);
impl FFL_R {
    #[inline(always)]
    pub(crate) fn new(bits: u8) -> Self {
        FFL_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FFL_R {
    type Target = crate::FieldReader<u8, u8>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FGI` reader - Rx FIFO Get Index"]
pub struct FGI_R(crate::FieldReader<u8, u8>);
impl FGI_R {
    #[inline(always)]
    pub(crate) fn new(bits: u8) -> Self {
        FGI_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FGI_R {
    type Target = crate::FieldReader<u8, u8>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FPI` reader - Rx FIFO Put Index"]
pub struct FPI_R(crate::FieldReader<u8, u8>);
impl FPI_R {
    #[inline(always)]
    pub(crate) fn new(bits: u8) -> Self {
        FPI_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FPI_R {
    type Target = crate::FieldReader<u8, u8>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `FF` reader - Rx FIFO Full"]
pub struct FF_R(crate::FieldReader<bool, bool>);
impl FF_R {
    #[inline(always)]
    pub(crate) fn new(bits: bool) -> Self {
        FF_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for FF_R {
    type Target = crate::FieldReader<bool, bool>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[doc = "Field `RFL` reader - Rx FIFO Message Lost"]
pub struct RFL_R(crate::FieldReader<bool, bool>);
impl RFL_R {
    #[inline(always)]
    pub(crate) fn new(bits: bool) -> Self {
        RFL_R(crate::FieldReader::new(bits))
    }
}
impl core::ops::Deref for RFL_R {
    type Target = crate::FieldReader<bool, bool>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl R {
    #[doc = "Bits 0:6 - Rx FIFO Fill Level"]
    #[inline(always)]
    pub fn ffl(&self) -> FFL_R {
        FFL_R::new((self.bits & 0x7f) as u8)
    }
    #[doc = "Bits 8:13 - Rx FIFO Get Index"]
    #[inline(always)]
    pub fn fgi(&self) -> FGI_R {
        FGI_R::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bits 16:21 - Rx FIFO Put Index"]
    #[inline(always)]
    pub fn fpi(&self) -> FPI_R {
        FPI_R::new(((self.bits >> 16) & 0x3f) as u8)
    }
    #[doc = "Bit 24 - Rx FIFO Full"]
    #[inline(always)]
    pub fn ff(&self) -> FF_R {
        FF_R::new(((self.bits >> 24) & 0x01) != 0)
    }
    #[doc = "Bit 25 - Rx FIFO Message Lost"]
    #[inline(always)]
    pub fn rfl(&self) -> RFL_R {
        RFL_R::new(((self.bits >> 25) & 0x01) != 0)
    }
}
#[doc = "Rx FIFO Status\n\nThis register you can [`read`](crate::reg::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [rxfs](index.html) module"]
pub struct RXFS_SPEC;
impl crate::RegisterSpec for RXFS_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [rxfs::R](R) reader structure"]
impl crate::Readable for RXFS_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets RXFS to value 0"]
impl crate::Resettable for RXFS_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}

#[doc = "Register `RXFS` reader"]
pub type R = crate::R<RXFS_SPEC>;
#[doc = "Field `FFL` reader - Rx FIFO Fill Level"]
pub type FFL_R = crate::FieldReader;
#[doc = "Field `FGI` reader - Rx FIFO Get Index"]
pub type FGI_R = crate::FieldReader;
#[doc = "Field `FPI` reader - Rx FIFO Put Index"]
pub type FPI_R = crate::FieldReader;
#[doc = "Field `FF` reader - Rx FIFO Full"]
pub type FF_R = crate::BitReader;
#[doc = "Field `RFL` reader - Rx FIFO Message Lost"]
pub type RFL_R = crate::BitReader;
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
        FF_R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Rx FIFO Message Lost"]
    #[inline(always)]
    pub fn rfl(&self) -> RFL_R {
        RFL_R::new(((self.bits >> 25) & 1) != 0)
    }
}
#[doc = "Rx FIFO Status\n\nYou can [`read`](crate::reg::generic::Reg::read) this register and get [`rxfs::R`](R).  See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RXFS_SPEC;
impl crate::RegisterSpec for RXFS_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rxfs::R`](R) reader structure"]
impl crate::Readable for RXFS_SPEC {}
#[doc = "`reset()` method sets RXFS to value 0"]
impl crate::Resettable for RXFS_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
